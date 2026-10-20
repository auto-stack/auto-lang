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
//! 寿命合同（V-02/V-05，含 R1/R2 复审修复）：
//! - 请求 deadline 覆盖**整次往返**：帧写入（writer 线程分片写 + 就绪
//!   轮询）与回答等待共用同一绝对截止（F-816-R1-01）。
//! - **任何返回路径都不带活的自有线程**（F-816-R2-06）：超时/关闭先请求
//!   停止、终止 owned 树、确认 reader/writer 真停（poll 轮询 + stop 标志
//!   + join）之后才返回；`Timeout` 仅在取消成功且收尾完成后给出。
//! - 宿主根退出后先终止 owned 后代；非拥有方持管道导致收尾不能完成时
//!   具名 [`ClientError::PipesHeld`]，不谎报、不悬挂。
//! - 控制流只解析帧且**先 drain 已缓冲内容**；坏帧/坏前缀在任何消费
//!   路径（请求/EOF 关闭/超时收尾）都不被吞掉（F-816-R1-03/R2-03）。
//! - 优雅关闭（EOF）按宿主合同期待退出码 0，宽限到期则 cancel 收尸。
//!
//! 边界（§5.3）：不静态链接任何工具实现（auto-ui-tools 等）——只与真实
//! 进程说话；不做工具发现/隐式搜索（T-04），调用方给出显式二进制路径；
//! 日志只从 stderr 读，控制流（stdout）出现任何非帧字节即具名
//! [`ClientError::StreamViolation`]，绝不静默解析。

use std::ffi::OsString;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use auto_tool_execution::{
    spawn as spawn_owned, ExecutionSpec, OwnedProcess, PipePoll, StdioPolicy, TreeScope,
    PIPE_POLL_SLICE,
};
use auto_tool_protocol::framing::{parse_frame, FrameError, FrameParse};
use auto_tool_protocol::message::{ClientBody, ClientEnvelope, ServerEnvelope, WireRequest};
use auto_tool_protocol::{
    Budgets, ToolDescriptor, ToolError, ToolEvent, ValidateProjectReport, PROTOCOL_VERSION,
};

// 消费方经本 crate 组装 spec/读结果，底层设施类型在此再导出，消费闭包
// 不需要直接依赖 auto-tool-execution。
pub use auto_tool_execution::ExitKind;

/// reader/writer 收尾的拆除宽限：stop 生效延迟 ≤ 轮询切片，正常远小于
/// 本值；超时即管道被非拥有进程持有（F-816-R1/R2-06）。
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
    /// stdin writer 失败（管道断裂/预算拒绝后致命错误），连接不可再用。
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
    /// 控制流出现非帧字节/坏帧：协议违规，流不可再用。**任何**消费路径
    /// （请求/EOF 关闭/超时收尾）都不得吞掉（F-816-R2-03）。
    StreamViolation(FrameError),
    /// 宿主在回答前关闭了控制流（含 EOF 夹带半帧——请求路径视角）。
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
    /// 线程确认真停之后才返回本错误；`limit` 为到期的时限。
    Timeout {
        id: u64,
        limit: Duration,
    },
    /// 宿主根进程已退出且 owned 树已终止，但管道写端仍被**非本次拥有**
    /// 的进程持有：reader/writer 在宽限内收不到完成，按具名失败返回
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
/// 非恐慌退出路径（含被停止）的最后一条消息；通道在无 `Done` 的情况下
/// 断开 = 线程 panic（F-816-R1-06）。
#[derive(Debug)]
enum FromReader {
    Frame(Box<ServerEnvelope>),
    /// 已解析的坏帧/坏前缀（TooLarge、非 JSON、非类型化信封）。
    Violation(FrameError),
    /// 控制流在半帧处结束。**关闭路径**（宿主自然退出）这是宿主违规；
    /// 超时/取消路径（我方 kill 截断在途合法回复）属拆除噪声——由消费
    /// 方按语境决定（F-816-R2-03）。
    TruncatedEof {
        trailing: usize,
    },
    Eof,
    Done,
}

/// 与一个真实工具宿主进程的连接。
pub struct ToolClient {
    proc: OwnedProcess,
    jobs_tx: Option<Sender<serde_json::Value>>,
    writer_ack: Receiver<Result<(), FrameError>>,
    writer_done: Receiver<Result<(), std::io::Error>>,
    writer_stop: Arc<AtomicBool>,
    rx: Receiver<FromReader>,
    reader_stop: Arc<AtomicBool>,
    reader_closed: Arc<AtomicBool>,
    logs: Arc<Mutex<Vec<u8>>>,
    stderr_done: Receiver<Result<(), std::io::Error>>,
    stderr_stop: Arc<AtomicBool>,
    stderr_closed: Arc<AtomicBool>,
    next_id: u64,
    spec: ToolClientSpec,
    broken: Option<BrokenReason>,
    collected: bool,
    writer_done_seen: bool,
}

impl ToolClient {
    /// spawn 真实宿主进程并接线控制/日志流。stdin 写入由内部 writer
    /// 线程拥有（分片写 + 就绪轮询：管道写阻塞不越过调用方 deadline，
    /// F-816-R1-01）；三条线程都可被设施停止（R2-06）。
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

        // ---- stdin writer 线程 ----
        let stdin = proc
            .take_stdin()
            .ok_or_else(|| ClientError::InvalidSpec("tool host stdin was not piped".into()))?;
        let (jobs_tx, jobs_rx) = std::sync::mpsc::channel::<serde_json::Value>();
        let (ack_tx, writer_ack) = std::sync::mpsc::channel::<Result<(), FrameError>>();
        let (wdone_tx, writer_done) = std::sync::mpsc::channel::<Result<(), std::io::Error>>();
        let writer_stop = Arc::new(AtomicBool::new(false));
        let wflag = Arc::clone(&writer_stop);
        let budget = spec.frame_budget;
        std::thread::spawn(move || {
            let mut stdin = stdin;
            let mut fatal: Option<std::io::Error> = None;
            loop {
                let job = match jobs_rx.recv_timeout(PIPE_POLL_SLICE) {
                    Ok(job) => job,
                    Err(RecvTimeoutError::Timeout) => {
                        if wflag.load(Ordering::Relaxed) {
                            break;
                        }
                        continue;
                    }
                    Err(RecvTimeoutError::Disconnected) => break,
                };
                let result = write_frame_chunked(&mut stdin, &job, budget);
                // 仅真实写出失败(Io)终结连接;预算/序列化拒绝发生在任何
                // 字节写出之前,流保持可用。
                let fatal_io = matches!(&result, Err(FrameError::Io(_)));
                let _ = ack_tx.send(result);
                if fatal_io {
                    fatal = Some(std::io::Error::other("stdin writer failed"));
                    break;
                }
            }
            // 通道关闭（调用方收尾）或致命写失败：drop stdin → 宿主见 EOF。
            drop(stdin);
            let _ = wdone_tx.send(Ok(()));
            let _ = fatal; // 致命性已随逐写回执传播;done 只承担"线程已停"。
        });

        // ---- stdout 控制流 reader 线程 ----
        let (tx, rx) = std::sync::mpsc::channel::<FromReader>();
        let reader_stop = Arc::new(AtomicBool::new(false));
        let reader_closed = Arc::new(AtomicBool::new(false));
        let (rstop, rclosed) = (Arc::clone(&reader_stop), Arc::clone(&reader_closed));
        let has_stdout = proc
            .take_stdout()
            .map(|out| spawn_stdout_reader(out, tx, budget, rstop, rclosed))
            .is_some();
        if !has_stdout {
            return Err(ClientError::InvalidSpec(
                "tool host stdout was not piped".into(),
            ));
        }

        // ---- stderr 日志 reader 线程（结束方式具名：EOF 或 I/O 错误）----
        let logs = Arc::new(Mutex::new(Vec::new()));
        let (sdone_tx, stderr_done) = std::sync::mpsc::channel::<Result<(), std::io::Error>>();
        let stderr_stop = Arc::new(AtomicBool::new(false));
        let stderr_closed = Arc::new(AtomicBool::new(false));
        let (sstop, sclosed, sink) = (
            Arc::clone(&stderr_stop),
            Arc::clone(&stderr_closed),
            Arc::clone(&logs),
        );
        if let Some(err) = proc.take_stderr() {
            std::thread::spawn(move || {
                let end = stderr_session(err, &sink, &sstop, &sclosed);
                let _ = sdone_tx.send(end);
            });
        } else {
            let _ = sdone_tx.send(Ok(()));
        }
        Ok(Self {
            proc,
            jobs_tx: Some(jobs_tx),
            writer_ack,
            writer_done,
            writer_stop,
            rx,
            reader_stop,
            reader_closed,
            logs,
            stderr_done,
            stderr_stop,
            stderr_closed,
            next_id: 1,
            spec,
            broken: None,
            collected: false,
            writer_done_seen: false,
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
    /// `started+limit` 是本次请求的唯一截止；写阻塞到截止即走超时路径。
    fn send_deadline(
        &mut self,
        json: &serde_json::Value,
        started: Instant,
        limit: Option<Duration>,
        id: u64,
    ) -> Result<(), ClientError> {
        let tx = self
            .jobs_tx
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
            FromReader::TruncatedEof { .. } | FromReader::Eof | FromReader::Done => {
                Err(ClientError::StreamClosed { id })
            }
        }
    }

    /// deadline 到期路径（F-816-R1-01/02、R2-03/06）：先结束 writer、终止
    /// owned 树（解除任何管道阻塞）、请求 reader 停止并确认**真停**；随后
    /// 取消失败、stderr 读错、缓冲坏帧具名传播，全部完成才返回 Timeout。
    fn abort_for_timeout(&mut self, id: u64, limit: Duration) -> ClientError {
        self.writer_stop.store(true, Ordering::Relaxed);
        drop(self.jobs_tx.take());
        let cancel_result = self.proc.cancel();
        self.reader_closed.store(true, Ordering::Relaxed);
        self.stderr_closed.store(true, Ordering::Relaxed);
        self.reader_stop.store(true, Ordering::Relaxed);
        self.stderr_stop.store(true, Ordering::Relaxed);
        self.broken = Some(BrokenReason::TimedOut);
        let drained = self.drain_until_done(TEARDOWN_GRACE, false);
        if let Err(source) = cancel_result {
            return ClientError::Io {
                stage: "cancel after timeout",
                source: Box::new(source),
            };
        }
        if let Err(e) = drained {
            return e;
        }
        ClientError::Timeout { id, limit }
    }

    /// 收尾 drain：等待 writer/stderr/stdout 三条线程的完成标记。
    ///
    /// `natural_exit` = 宿主自然退出（关闭路径）：缓冲的坏帧（含 EOF 夹
    /// 半帧）按控制流违规保留在最终结果里；false（超时/取消路径）：
    /// 我方终止造成的 `Violation(Io)`（管道拆除）与 `TruncatedEof`（kill
    /// 截断在途回复）属拆除噪声，其余坏帧仍具名传播（F-816-R2-03）。
    fn drain_until_done(&mut self, grace: Duration, natural_exit: bool) -> Result<(), ClientError> {
        if self.collected {
            return Ok(());
        }
        // 三条线程全部终结后才传播任一失败（F-816-R3-02）。
        let writer_err = if self.writer_done_seen {
            None
        } else {
            match self.writer_done.recv_timeout(grace) {
                Ok(_) => {
                    self.writer_done_seen = true;
                    None
                }
                Err(RecvTimeoutError::Timeout) => Some(ClientError::PipesHeld { after: grace }),
                Err(RecvTimeoutError::Disconnected) => Some(ClientError::Panicked {
                    which: "stdin writer",
                }),
            }
        };
        let stderr_err = match self.stderr_done.recv_timeout(grace) {
            Ok(Ok(())) => None,
            Ok(Err(source)) => Some(ClientError::Io {
                stage: "stderr reader",
                source: Box::new(source),
            }),
            Err(RecvTimeoutError::Timeout) => Some(ClientError::PipesHeld { after: grace }),
            Err(RecvTimeoutError::Disconnected) => Some(ClientError::Panicked {
                which: "stderr reader",
            }),
        };
        // stdout：drain 到 Done；途经的坏帧按语境定性（记下、不中断
        // drain——Done 之后统一传播，保证错误返回时线程已终结）。
        let mut stdout_err: Option<ClientError> = None;
        let deadline = Instant::now() + grace;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match self.rx.recv_timeout(remaining) {
                Ok(FromReader::Done) => break,
                Ok(FromReader::Violation(e)) => {
                    let teardown = !natural_exit && matches!(e, FrameError::Io(_));
                    if !teardown && stdout_err.is_none() {
                        stdout_err = Some(ClientError::StreamViolation(e));
                    }
                }
                Ok(FromReader::TruncatedEof { trailing }) => {
                    if natural_exit && stdout_err.is_none() {
                        stdout_err = Some(ClientError::StreamViolation(FrameError::Malformed {
                            reason: format!(
                                "control stream closed mid-frame ({trailing} trailing bytes)"
                            ),
                        }));
                    }
                    // 超时/取消路径：我方 kill 截断在途合法回复——噪声。
                }
                Ok(_) => continue,
                Err(RecvTimeoutError::Timeout) => {
                    stdout_err = Some(ClientError::PipesHeld { after: grace });
                    break;
                }
                Err(RecvTimeoutError::Disconnected) => {
                    stdout_err = Some(ClientError::Panicked {
                        which: "stdout reader",
                    });
                    break;
                }
            }
        }
        self.collected = true;
        if let Some(e) = writer_err {
            return Err(e);
        }
        if let Some(e) = stderr_err {
            return Err(e);
        }
        if let Some(e) = stdout_err {
            self.broken = Some(BrokenReason::StreamViolated);
            return Err(e);
        }
        Ok(())
    }

    /// 请求全部自有线程停止并标记 owned 写端全关（错误路径统一收尾用，
    /// F-816-R3-02——任何返回路径都不带活的自有线程）。
    fn stop_all(&self) {
        self.reader_closed.store(true, Ordering::Relaxed);
        self.stderr_closed.store(true, Ordering::Relaxed);
        self.reader_stop.store(true, Ordering::Relaxed);
        self.stderr_stop.store(true, Ordering::Relaxed);
        self.writer_stop.store(true, Ordering::Relaxed);
    }

    /// 优雅关闭：结束 writer（EOF）→ 宽限内等待宿主退出（合同期待码 0）；
    /// 根退出后先终止仍持管道的 owned 后代（F-816-R1-02），三条线程确
    /// 认真停；宽限到期则 cancel 收尸。返回退出分类与完整日志。
    pub fn shutdown(mut self) -> Result<Shutdown, ClientError> {
        let deadline = Instant::now() + self.spec.shutdown_timeout;
        // 1. 结束 writer（jobs 通道关闭 → 分片写完成 → drop stdin → EOF）。
        drop(self.jobs_tx.take());
        if !self.writer_done_seen {
            let writer_grace = deadline
                .saturating_duration_since(Instant::now())
                .min(TEARDOWN_GRACE);
            match self.writer_done.recv_timeout(writer_grace) {
                Ok(_) => self.writer_done_seen = true,
                Err(RecvTimeoutError::Timeout) => {
                    return Err(ClientError::PipesHeld {
                        after: writer_grace,
                    })
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(ClientError::Panicked {
                        which: "stdin writer",
                    })
                }
            }
        }
        // 2. 宿主退出（自然=真实分类保留;宽限尽=cancel）。
        let natural_exit;
        let status = match self
            .proc
            .wait_deadline(deadline.saturating_duration_since(Instant::now()))
        {
            Ok(Some(status)) => {
                natural_exit = true;
                if self.proc.tree_scope() == TreeScope::ProcessTree {
                    if let Err(source) = self.proc.terminate() {
                        self.stop_all();
                        return Err(ClientError::Io {
                            stage: "terminate after root exit",
                            source: Box::new(source),
                        });
                    }
                }
                status
            }
            Ok(None) => {
                natural_exit = false;
                match self.proc.cancel() {
                    Ok(status) => status,
                    Err(source) => {
                        self.stop_all();
                        return Err(ClientError::Io {
                            stage: "cancel after shutdown grace",
                            source: Box::new(source),
                        });
                    }
                }
            }
            Err(source) => {
                self.stop_all();
                return Err(ClientError::Io {
                    stage: "wait for host shutdown",
                    source: Box::new(source),
                });
            }
        };
        // 3. owned 写端已全关 → reader 的 EOF 判定;停止并确认三条线程。
        self.reader_closed.store(true, Ordering::Relaxed);
        self.stderr_closed.store(true, Ordering::Relaxed);
        self.reader_stop.store(true, Ordering::Relaxed);
        self.stderr_stop.store(true, Ordering::Relaxed);
        self.drain_until_done(TEARDOWN_GRACE, natural_exit)?;
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

/// 序列化并预算检查一帧（不写）;分片写在 [`write_frame_chunked`] 内。
fn frame_payload(json: &serde_json::Value, max_frame_bytes: usize) -> Result<Vec<u8>, FrameError> {
    let payload = serde_json::to_vec(json).map_err(|e| FrameError::Malformed {
        reason: format!("json serialization failed: {e}"),
    })?;
    if payload.len() > max_frame_bytes {
        return Err(FrameError::TooLarge {
            limit: max_frame_bytes,
            observed: payload.len() as u64,
        });
    }
    let mut bytes = (payload.len() as u32).to_be_bytes().to_vec();
    bytes.extend_from_slice(&payload);
    Ok(bytes)
}

/// 分片写一帧：16KiB 分片 + 就绪轮询 + stop 检查（由调用方的 writer
/// 循环持有 stop 语义,本函数只做有界分片写;预算拒绝发生在写出前）。
fn write_frame_chunked<W: std::io::Write + PipePoll>(
    w: &mut W,
    json: &serde_json::Value,
    max_frame_bytes: usize,
) -> Result<(), FrameError> {
    let bytes = frame_payload(json, max_frame_bytes)?;
    let mut written = 0usize;
    while written < bytes.len() {
        if !w.poll_ready(PIPE_POLL_SLICE) {
            continue;
        }
        let upper = (written + 16 * 1024).min(bytes.len());
        match w.write(&bytes[written..upper]) {
            Ok(0) => {
                return Err(FrameError::Io(std::io::Error::from(
                    std::io::ErrorKind::WriteZero,
                )))
            }
            Ok(n) => written += n,
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::Interrupted =>
            {
                continue
            }
            Err(e) => return Err(FrameError::Io(e)),
        }
    }
    Ok(())
}

/// stderr 日志 reader 会话：结束方式具名（干净 EOF / I/O 错误 / 停止）。
/// stop 语义=先排空已可读日志再退出（F-816-R3-01：日志不得缺字）。
fn stderr_session<R>(
    mut r: R,
    sink: &Mutex<Vec<u8>>,
    stop: &AtomicBool,
    owned_closed: &AtomicBool,
) -> Result<(), std::io::Error>
where
    R: Read + PipePoll,
{
    let mut chunk = [0u8; 8192];
    loop {
        if stop.load(Ordering::Relaxed) {
            // F-816-R3-01: 先排空已可读日志再退出——就地写入 sink。
            let cap = Instant::now() + DRAIN_CAP;
            while Instant::now() < cap {
                if !r.poll_ready(Duration::from_millis(0)) {
                    break;
                }
                match r.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => sink.lock().unwrap().extend_from_slice(&chunk[..n]),
                    Err(_) => break,
                }
            }
            return Ok(());
        }
        if !r.poll_ready(PIPE_POLL_SLICE) {
            if owned_closed.load(Ordering::Relaxed) {
                return Ok(());
            }
            continue;
        }
        match r.read(&mut chunk) {
            Ok(0) => return Ok(()),
            Ok(n) => sink.lock().unwrap().extend_from_slice(&chunk[..n]),
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::Interrupted =>
            {
                continue
            }
            Err(e) => return Err(e),
        }
    }
}

/// 具体类型直接持有（避免 trait object 的轮询转发;ChildStdout 在两
/// 平台都实现 [`PipePoll`]）。
fn spawn_stdout_reader<R>(
    r: R,
    tx: Sender<FromReader>,
    budget: usize,
    stop: Arc<AtomicBool>,
    owned_closed: Arc<AtomicBool>,
) -> std::thread::JoinHandle<()>
where
    R: Read + PipePoll + Send + 'static,
{
    std::thread::spawn(move || {
        let mut r = r;
        reader_session(&mut r, &tx, budget, &stop, &owned_closed);
    })
}

/// 控制流 reader 会话：[`reader_run`] 之后以 `Done` 收尾（panic 时不发
/// ——通道断开可判）。
fn reader_session<R>(
    r: &mut R,
    tx: &Sender<FromReader>,
    max_frame_bytes: usize,
    stop: &AtomicBool,
    owned_closed: &AtomicBool,
) where
    R: Read + PipePoll,
{
    reader_run(r, tx, max_frame_bytes, stop, owned_closed);
    let _ = tx.send(FromReader::Done);
}

/// 排空当前已可读内容到缓冲（F-816-R3-01：stop/收尾=先排空再退出，
/// 已到达数据不得丢弃）。有界：管道空即止；DRAIN_CAP 防御仍活跃的
/// 写入方无限供数。
fn drain_readable_into<R: Read + PipePoll>(r: &mut R, buffer: &mut Vec<u8>) {
    let cap = Instant::now() + DRAIN_CAP;
    let mut chunk = [0u8; 8192];
    while Instant::now() < cap {
        if !r.poll_ready(Duration::from_millis(0)) {
            break;
        }
        match r.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => buffer.extend_from_slice(&chunk[..n]),
            Err(_) => break,
        }
    }
}

/// 排空上限（防御性；owned 树已终止时数据有限）。
const DRAIN_CAP: Duration = Duration::from_millis(500);

/// 缓冲内可解析内容的 drain（完整帧发送、坏帧/坏前缀具名违规）。
/// 返回 false = 终结（违规已发或接收端已弃）——调用方随即退出。
fn parse_drain(
    buffer: &mut Vec<u8>,
    pos: &mut usize,
    tx: &Sender<FromReader>,
    max_frame_bytes: usize,
) -> bool {
    loop {
        match parse_frame(&buffer[*pos..], max_frame_bytes) {
            FrameParse::Complete { message, consumed } => {
                *pos += consumed;
                match serde_json::from_value::<ServerEnvelope>(message) {
                    Ok(envelope) => {
                        if tx.send(FromReader::Frame(Box::new(envelope))).is_err() {
                            return false;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(FromReader::Violation(FrameError::Malformed {
                            reason: format!("reply is not a typed envelope: {e}"),
                        }));
                        return false;
                    }
                }
                if *pos > 0 {
                    buffer.drain(..*pos);
                    *pos = 0;
                }
            }
            FrameParse::Incomplete { .. } => return true,
            FrameParse::Malformed { reason } => {
                let _ = tx.send(FromReader::Violation(FrameError::Malformed { reason }));
                return false;
            }
            FrameParse::TooLarge { limit, observed } => {
                let _ = tx.send(FromReader::Violation(FrameError::TooLarge {
                    limit,
                    observed,
                }));
                return false;
            }
        }
    }
}

/// 控制流 reader 主体（泛化 `Read`+`PipePoll` 以便故障注入单测）：
/// - **先 drain 缓冲内全部可解析内容**（完整帧与坏前缀都不等待新字节，
///   仅 Incomplete 才读取——F-816-R1-03）；
/// - 就绪轮询 + 停止标志：线程可被设施停止（F-816-R2-06），且 stop
///   语义=**先排空已可读内容并解析再退出**——已到达的帧/违规不得因
///   stop 丢弃（F-816-R3-01）;
/// - 任何非帧字节/坏帧/超预算帧都是具名违规且终结流;EOF 夹半帧记为
///   [`FromReader::TruncatedEof`]（消费方按语境定性）。
fn reader_run<R>(
    r: &mut R,
    tx: &Sender<FromReader>,
    max_frame_bytes: usize,
    stop: &AtomicBool,
    owned_closed: &AtomicBool,
) where
    R: Read + PipePoll,
{
    let mut buffer = Vec::new();
    let mut pos = 0usize;
    let mut chunk = [0u8; 8192];
    loop {
        if !parse_drain(&mut buffer, &mut pos, tx, max_frame_bytes) {
            return;
        }
        if stop.load(Ordering::Relaxed) {
            // F-816-R3-01: 先排空 OS 管道中已到达的字节并解析,再退出。
            drain_readable_into(r, &mut buffer);
            parse_drain(&mut buffer, &mut pos, tx, max_frame_bytes);
            return;
        }
        if !r.poll_ready(PIPE_POLL_SLICE) {
            if owned_closed.load(Ordering::Relaxed) {
                // owned 写端全关且无数据:EOF(或半帧截断)。
                if buffer.len() > pos {
                    let _ = tx.send(FromReader::TruncatedEof {
                        trailing: buffer.len() - pos,
                    });
                } else {
                    let _ = tx.send(FromReader::Eof);
                }
                return;
            }
            continue;
        }
        match r.read(&mut chunk) {
            Ok(0) => {
                if buffer.len() > pos {
                    let _ = tx.send(FromReader::TruncatedEof {
                        trailing: buffer.len() - pos,
                    });
                } else {
                    let _ = tx.send(FromReader::Eof);
                }
                return;
            }
            Ok(n) => buffer.extend_from_slice(&chunk[..n]),
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::Interrupted =>
            {
                continue
            }
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
        let e = ClientError::from(auto_tool_execution::ExecutionError::PipesHeld {
            after: Duration::from_millis(5),
        });
        assert!(e.to_string().contains("execution facility"));
    }

    // ---- 故障注入面（F-816-R1-03/06、R2-03/06 永久回归，无 sleep）----

    /// 预置分块的假 Read：依次交付 `chunks`，之后按 `tail` 结束。
    /// `idle_ready`=分块耗尽后是否仍报就绪（true 模拟"恒就绪"直接进
    /// read——Eof/Err 立即返回;false 模拟空管道轮询——stop 可在
    /// 不进 read 的情况下生效）。
    struct Chunked {
        chunks: Vec<Vec<u8>>,
        tail: Tail,
        idle_ready: bool,
    }

    enum Tail {
        Eof,
        Err(&'static str),
        /// 永不结束(模拟管道写端被非拥有进程持有)。
        Block,
    }

    impl Read for Chunked {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if let Some(chunk) = self.chunks.first() {
                let n = chunk.len().min(buf.len());
                buf[..n].copy_from_slice(&chunk[..n]);
                self.chunks.remove(0);
                return Ok(n);
            }
            match self.tail {
                Tail::Eof => Ok(0),
                Tail::Err(msg) => Err(std::io::Error::other(msg)),
                Tail::Block => loop {
                    std::thread::sleep(Duration::from_secs(3600));
                },
            }
        }
    }

    impl PipePoll for Chunked {
        fn poll_ready(&self, _slice: Duration) -> bool {
            !self.chunks.is_empty() || self.idle_ready
        }
    }

    fn flags() -> (Arc<AtomicBool>, Arc<AtomicBool>) {
        (
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicBool::new(false)),
        )
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
        let (stop, closed) = flags();
        let good = frame_bytes(&typed_error_envelope(1));
        let mut combined = good;
        combined.extend_from_slice(&[0xff, 0xff, 0xff, 0xff]);
        let mut reader = Chunked {
            idle_ready: true,
            chunks: vec![combined],
            tail: Tail::Err("read after buffered content — drain regression"),
        };
        reader_session(
            &mut reader,
            &tx,
            Budgets::contract().max_frame_bytes,
            &stop,
            &closed,
        );
        match rx.try_recv().unwrap() {
            FromReader::Frame(env) => assert_eq!(env.id, 1),
            other => panic!("expected buffered frame first, got {other:?}"),
        }
        match rx.try_recv().unwrap() {
            FromReader::Violation(FrameError::TooLarge { observed, .. }) => {
                assert_eq!(observed, 0xffff_ffff);
            }
            other => panic!("expected buffered TooLarge, got {other:?}"),
        }
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Done));
        assert!(rx.try_recv().is_err());
    }

    /// F-816-R1-03 相邻面：粘包（一帧读内多帧）全部 drain，再 EOF。
    #[test]
    fn coalesced_frames_all_drained_before_eof() {
        let (tx, rx) = std::sync::mpsc::channel::<FromReader>();
        let (stop, closed) = flags();
        let mut combined = frame_bytes(&typed_error_envelope(1));
        combined.extend_from_slice(&frame_bytes(
            &serde_json::to_value(ServerEnvelope::error(
                2,
                ToolError::not_cancellable("describe"),
            ))
            .unwrap(),
        ));
        let mut reader = Chunked {
            idle_ready: true,
            chunks: vec![combined],
            tail: Tail::Eof,
        };
        reader_session(
            &mut reader,
            &tx,
            Budgets::contract().max_frame_bytes,
            &stop,
            &closed,
        );
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Frame(_)));
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Frame(_)));
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Eof));
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Done));
    }

    /// F-816-R1-06：reader 读错具名传播（不是静默 EOF）。
    #[test]
    fn reader_io_error_surfaces_as_violation() {
        let (tx, rx) = std::sync::mpsc::channel::<FromReader>();
        let (stop, closed) = flags();
        let mut reader = Chunked {
            idle_ready: true,
            chunks: vec![(3u32).to_be_bytes().to_vec()],
            tail: Tail::Err("kernel pipe gone"),
        };
        reader_session(
            &mut reader,
            &tx,
            Budgets::contract().max_frame_bytes,
            &stop,
            &closed,
        );
        match rx.try_recv().unwrap() {
            FromReader::Violation(FrameError::Io(e)) => {
                assert!(e.to_string().contains("kernel pipe gone"))
            }
            other => panic!("expected Io violation, got {other:?}"),
        }
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Done));
    }

    /// EOF 夹带半帧 → `TruncatedEof`（关闭路径定性为违规的依据）。
    #[test]
    fn eof_with_partial_frame_is_truncated_eof() {
        let (tx, rx) = std::sync::mpsc::channel::<FromReader>();
        let (stop, closed) = flags();
        let mut partial = (64u32).to_be_bytes().to_vec();
        partial.extend_from_slice(b"{\"id\": 1");
        let mut reader = Chunked {
            idle_ready: true,
            chunks: vec![partial],
            tail: Tail::Eof,
        };
        reader_session(
            &mut reader,
            &tx,
            Budgets::contract().max_frame_bytes,
            &stop,
            &closed,
        );
        match rx.try_recv().unwrap() {
            FromReader::TruncatedEof { trailing } => assert!(trailing > 0),
            other => panic!("expected TruncatedEof, got {other:?}"),
        }
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Done));
    }

    /// 帧内 JSON 合法但不符合信封类型 → 具名"not a typed envelope"。
    #[test]
    fn untyped_envelope_is_named_violation() {
        let (tx, rx) = std::sync::mpsc::channel::<FromReader>();
        let (stop, closed) = flags();
        let mut reader = Chunked {
            idle_ready: true,
            chunks: vec![frame_bytes(&serde_json::json!({"id": 1, "nope": true}))],
            tail: Tail::Eof,
        };
        reader_session(
            &mut reader,
            &tx,
            Budgets::contract().max_frame_bytes,
            &stop,
            &closed,
        );
        match rx.try_recv().unwrap() {
            FromReader::Violation(FrameError::Malformed { reason }) => {
                assert!(reason.contains("typed envelope"), "{reason}")
            }
            other => panic!("expected typed-envelope violation, got {other:?}"),
        }
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Done));
    }

    /// F-816-R3-01：stop 语义=先排空并解析已到达帧——预置两帧+先行置
    /// stop，两帧都必须送达（确定性，无 sleep/无负载依赖）。
    #[test]
    fn stop_drains_preset_frames_before_stopping() {
        let (tx, rx) = std::sync::mpsc::channel::<FromReader>();
        let (stop, closed) = flags();
        stop.store(true, Ordering::Relaxed);
        let mut reader = Chunked {
            idle_ready: true,
            chunks: vec![
                frame_bytes(&typed_error_envelope(1)),
                frame_bytes(
                    &serde_json::to_value(ServerEnvelope::error(
                        2,
                        ToolError::not_cancellable("describe"),
                    ))
                    .unwrap(),
                ),
            ],
            tail: Tail::Eof,
        };
        reader_session(
            &mut reader,
            &tx,
            Budgets::contract().max_frame_bytes,
            &stop,
            &closed,
        );
        match rx.try_recv().unwrap() {
            FromReader::Frame(env) => assert_eq!(env.id, 1),
            other => panic!("frame 1 must survive stop, got {other:?}"),
        }
        match rx.try_recv().unwrap() {
            FromReader::Frame(env) => assert_eq!(env.id, 2),
            other => panic!("frame 2 must survive stop, got {other:?}"),
        }
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Done));
    }

    /// F-816-R3-01（stderr 面）：stop 先排空已到达日志——预置日志行+
    /// 先行置 stop，sink 必须完整包含（R3 实机间歇缺字根因的确定性反例）。
    #[test]
    fn stderr_stop_drains_preset_bytes() {
        let (stop, closed) = flags();
        stop.store(true, Ordering::Relaxed);
        let sink = Mutex::new(Vec::new());
        let reader = Chunked {
            idle_ready: true,
            chunks: vec![b"accepted request id=1 op=describe
"
            .to_vec()],
            tail: Tail::Eof,
        };
        let end = stderr_session(reader, &sink, &stop, &closed);
        assert!(end.is_ok());
        let logs = String::from_utf8(sink.into_inner().unwrap()).unwrap();
        assert!(logs.contains("accepted request id=1"), "logs: {logs:?}");
    }

    /// F-816-R2-06：stop 标志使 reader 真停（不依赖 EOF/数据到达）。
    #[test]
    fn stop_flag_stops_reader_without_more_reads() {
        let (tx, rx) = std::sync::mpsc::channel::<FromReader>();
        let (stop, closed) = flags();
        let mut reader = Chunked {
            idle_ready: false,
            chunks: vec![frame_bytes(&typed_error_envelope(1))],
            tail: Tail::Block,
        };
        // 先 drain 帧再置 stop:线程在 Block 读之前观察到 stop 即退。
        let stop_thread = Arc::clone(&stop);
        let handle = std::thread::spawn(move || {
            reader_session(
                &mut reader,
                &tx,
                Budgets::contract().max_frame_bytes,
                &stop_thread,
                &closed,
            )
        });
        // 等首帧被 drain(有界),随后置 stop。
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(FromReader::Frame(env)) => {
                    assert_eq!(env.id, 1);
                    break;
                }
                Ok(FromReader::Done) => panic!("reader stopped before the frame was drained"),
                Ok(other) => panic!("unexpected message {other:?}"),
                Err(RecvTimeoutError::Timeout) if Instant::now() < deadline => continue,
                Err(e) => panic!("channel error {e:?}"),
            }
        }
        stop.store(true, Ordering::Relaxed);
        handle.join().expect("reader thread stops after the flag");
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Done));
    }
}

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
use auto_tool_protocol::message::{
    ClientBody, ClientEnvelope, ServerBody, ServerEnvelope, WireRequest,
};
use auto_tool_protocol::report::BuildReport;
use auto_tool_protocol::{
    op_options, Budgets, GenerationReport, RunReport, ToolDescriptor, ToolError, ToolEvent,
    ValidateProjectReport, PROTOCOL_VERSION,
};

// 消费方经本 crate 组装 spec/读结果，底层设施类型在此再导出，消费闭包
// 不需要直接依赖 auto-tool-execution。
pub use auto_tool_execution::ExitKind;

pub mod discovery;
pub use discovery::{discover, resolve_existing, DiscoveredTool, DiscoveryError, DiscoverySpec};

/// reader/writer 收尾的拆除宽限：stop 生效延迟 ≤ 轮询切片，正常远小于
/// 本值；超时即管道被非拥有进程持有（F-816-R1/R2-06）。
const TEARDOWN_GRACE: Duration = Duration::from_secs(3);

/// 一次工具宿主连接的完整请求。
#[derive(Clone)]
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
    /// 活日志回调：宿主 stderr 每排空一段即同步调用（None=只收集）。
    /// 长运行操作（run_ui dev-server）的人读 parity 通道——转发到本进程
    /// stderr 即恢复旧直跑形态的实时输出。回调里不得再调用本客户端。
    pub live_log: Option<LiveLogFn>,
}

/// 活日志回调类型（宿主 stderr 段级转发；见 [`ToolClientSpec::live_log`]）。
pub type LiveLogFn = Arc<dyn Fn(&[u8]) + Send + Sync>;

/// generate_ui 的执行选项（PLAN-816 T-05 R1；与协议 op 级字段一一对应，
/// 缺省=旧发布行为）。`output`/`project`=旧 `gen -o/-p`；
/// `delivery_root`=792 显式交付（仅 rust）；`strict`=旧 build 默认严格。
#[derive(Debug, Clone, Default)]
pub struct GenerateUiOpts {
    pub output: Option<std::path::PathBuf>,
    pub project: Option<bool>,
    pub delivery_root: Option<std::path::PathBuf>,
    pub strict: Option<bool>,
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
            live_log: None,
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
    /// M815-B 扩展回执（GenerateOk/RunOk/BuildOk/Cancelled/InputAck）——
    /// 本客户端当期消费面未发起对应操作，原文保留；落到未期待它的调用
    /// 侧由既有 `other` 臂以 [`ClientError::MalformedReply`] 具名拒绝。
    Other(ServerBody),
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
    /// 交互 stdin 泵在回收宽限内未确认真停（F-816-T05-R2-03）：取消已
    /// 按切片重发仍未见线程结束；泵句柄保留在客户端槽内（residue），
    /// shutdown 会再有界重试——返回本错误时**不**声称泵已回收。
    PumpStuck {
        after: Duration,
    },
    /// 请求携带的 op 级执行选项未被宿主 describe 广播兑现
    /// （F-816-T05-R2-04）：在任何帧写出（=任何副作用）之前拒绝。
    /// 旧 v1 工具对新增字段是 serde 静默忽略——"可解码"不等于"已执行"，
    /// 不能把选项被丢掉的生成当成功。
    UnsupportedOption {
        operation: &'static str,
        option: String,
        tool_version: String,
    },
    /// 宿主广播了选项、也回了成功报告，但**报告落点与请求选项不符**
    /// （F-816-T05-R2-04 端到端核实）：典型为旧 wire 中间人剥掉请求
    /// 选项后工具按默认布局生成——describe 广播拦不住说谎的中继，
    /// 落点核对是最后一道保险，具名失败而非静默接受被丢选项的产物。
    OptionIgnored {
        operation: &'static str,
        option: &'static str,
        requested: String,
        reported: String,
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
            ClientError::PumpStuck { after } => write!(
                f,
                "interactive stdin pump did not stop within {after:?} despite \
                 re-issued cancellations; a residue reader may remain"
            ),
            ClientError::UnsupportedOption {
                operation,
                option,
                tool_version,
            } => write!(
                f,
                "tool (version {tool_version}) does not advertise `{operation}` option \
                 `{option}`; request NOT sent — upgrade the tool or drop the option \
                 (an old v1 host silently ignores unknown options)"
            ),
            ClientError::OptionIgnored {
                operation,
                option,
                requested,
                reported,
            } => write!(
                f,
                "`{operation}` option `{option}` was silently ignored: requested \
                 {requested} but the report says artifacts landed under {reported}"
            ),
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

/// 交互 stdin 泵的回收归属（F-816-T05-R1-03）。
struct PumpHandle {
    stop: Arc<AtomicBool>,
    join: Option<std::thread::JoinHandle<()>>,
}

/// 单个泵句柄的有界回收（`ToolClient::reclaim_pump` 的可测内核，
/// F-816-T05-R2-03）：置停 → 等待期间每 5ms **重发**取消（覆盖就绪
/// 判定后才进入内核的挂起读——单发取消对"取消先于读入核"的时序无效）
/// → 真停后 join 收割。宽限尽=句柄放回 `pump.join` 并返回具名
/// `PumpStuck`（调用方保留归属供后续重试）；join panic=具名 `Panicked`。
fn reclaim_pump_handle(pump: &mut PumpHandle, grace: Duration) -> Result<(), ClientError> {
    pump.stop.store(true, Ordering::Relaxed);
    let Some(join) = pump.join.take() else {
        return Ok(()); // 无线程归属（先前已收割）
    };
    let deadline = std::time::Instant::now() + grace;
    while !join.is_finished() && std::time::Instant::now() < deadline {
        pump_io::cancel_stdin_read();
        std::thread::sleep(Duration::from_millis(5));
    }
    if !join.is_finished() {
        pump.join = Some(join); // residue：归属保留，不静默丢线程
        return Err(ClientError::PumpStuck { after: grace });
    }
    match join.join() {
        Ok(()) => Ok(()),
        Err(_) => Err(ClientError::Panicked {
            which: "stdin pump",
        }),
    }
}

#[cfg(windows)]
mod pump_io {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::Sender;
    use std::sync::Arc;

    extern "system" {
        fn GetStdHandle(n: u32) -> isize;
        fn WaitForSingleObject(h: isize, ms: u32) -> u32;
        fn ReadFile(h: isize, buf: *mut u8, n: u32, read: *mut u32, overlapped: *const u8) -> i32;
        fn CancelIoEx(h: isize, overlapped: *const u8) -> i32;
    }
    const STD_INPUT_HANDLE: u32 = 4294967286; // (u32)-10

    /// 就绪分片读宿主进程 stdin：50ms 可等待切片间检查 stop；字节经行
    /// 缓冲切成 Input 帧。停止侧 `cancel_stdin_read` 以 CancelIoEx 中止
    /// 挂起读（泵自持同一 std 句柄，取消可及——管道/控制台同机制）。
    pub fn pump_loop(
        op_id: u64,
        version: u32,
        tx: Sender<(u64, serde_json::Value)>,
        stop: Arc<AtomicBool>,
    ) {
        use auto_tool_protocol::message::{ClientBody, ClientEnvelope};
        unsafe {
            let h = GetStdHandle(STD_INPUT_HANDLE);
            if h <= 0 {
                return;
            }
            let mut line_buf: Vec<u8> = Vec::new();
            let mut chunk = [0u8; 1024];
            loop {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                if WaitForSingleObject(h, 50) != 0 {
                    continue; // timeout slice — stop re-checked
                }
                // 就绪→入核前复查停止（F-816-T05-R2-03：就绪判定与
                // ReadFile 入核之间的窗口内落地的停止不再消费输入；窗口
                // 尾部（复查后、入核前）由回收侧重发取消覆盖）。
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                let mut n = 0u32;
                if ReadFile(
                    h,
                    chunk.as_mut_ptr(),
                    chunk.len() as u32,
                    &mut n,
                    std::ptr::null(),
                ) == 0
                {
                    break; // error / cancelled read
                }
                if n == 0 {
                    break; // EOF
                }
                line_buf.extend_from_slice(&chunk[..n as usize]);
                while let Some(pos) = line_buf.iter().position(|&b| b == b'\n') {
                    let data: Vec<u8> = line_buf.drain(..=pos).collect();
                    let envelope = ClientEnvelope {
                        protocol_version: version,
                        id: op_id,
                        body: ClientBody::Input {
                            target_id: op_id,
                            data,
                        },
                    };
                    let Ok(json) = serde_json::to_value(&envelope) else {
                        return;
                    };
                    if tx.send((op_id, json)).is_err() {
                        return; // writer gone — teardown in progress
                    }
                }
            }
        }
    }

    /// 取消泵线程在 std 输入句柄上的挂起读（有界回收的一部分）。
    pub fn cancel_stdin_read() {
        unsafe {
            let h = GetStdHandle(STD_INPUT_HANDLE);
            if h > 0 {
                CancelIoEx(h, std::ptr::null());
            }
        }
    }
}

#[cfg(unix)]
mod pump_io {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::Sender;
    use std::sync::Arc;

    // 零依赖 FFI（F-816-T05-R2-01：先前直接引用 libc:: 而本 crate 无该
    // 依赖，Linux target check E0433。镜像 auto-tool-execution/unix_pg
    // 的显式 extern 声明风格；ABI 按 glibc/musl 与 macOS 的共同子集——
    // nfds_t=unsigned long(64 位平台 u64)、events/revents=short)。
    #[repr(C)]
    struct PollFd {
        fd: i32,
        events: i16,
        revents: i16,
    }
    const POLLIN: i16 = 0x001;
    extern "C" {
        fn poll(fds: *mut PollFd, nfds: u64, timeout_ms: i32) -> i32;
        fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
    }

    /// 就绪分片读（poll(2) 50ms 切片——与 Windows WaitForSingleObject
    /// 同语义：停止标志在切片间恒可观察，非 owned 持读者不能把泵钉死
    /// 在阻塞读上；镜像 815 R4 的双平台承诺）。
    pub fn pump_loop(
        op_id: u64,
        version: u32,
        tx: Sender<(u64, serde_json::Value)>,
        stop: Arc<AtomicBool>,
    ) {
        use auto_tool_protocol::message::{ClientBody, ClientEnvelope};
        let mut pollfd = PollFd {
            fd: 0,
            events: POLLIN,
            revents: 0,
        };
        let mut line_buf: Vec<u8> = Vec::new();
        let mut chunk = [0u8; 1024];
        loop {
            if stop.load(Ordering::Relaxed) {
                break;
            }
            let r = unsafe { poll(&mut pollfd, 1, 50) };
            if r <= 0 {
                continue; // timeout / signal — stop re-checked
            }
            // 就绪→入核前复查停止（F-816-T05-R2-03：就绪判定与读入核
            // 之间的窗口内落地的停止不再消费后续输入）。
            if stop.load(Ordering::Relaxed) {
                break;
            }
            let n = unsafe { read(0, chunk.as_mut_ptr(), chunk.len()) };
            if n <= 0 {
                break; // EOF / error
            }
            line_buf.extend_from_slice(&chunk[..n as usize]);
            while let Some(pos) = line_buf.iter().position(|&b| b == b'\n') {
                let data: Vec<u8> = line_buf.drain(..=pos).collect();
                let envelope = ClientEnvelope {
                    protocol_version: version,
                    id: op_id,
                    body: ClientBody::Input {
                        target_id: op_id,
                        data,
                    },
                };
                let Ok(json) = serde_json::to_value(&envelope) else {
                    return;
                };
                if tx.send((op_id, json)).is_err() {
                    return;
                }
            }
        }
    }

    /// Unix：poll 切片读天然可停（无不可取消的挂起读）——空实现保持
    /// 双平台调用面一致。
    pub fn cancel_stdin_read() {}
}

/// 与一个真实工具宿主进程的连接。
pub struct ToolClient {
    proc: OwnedProcess,
    jobs_tx: Option<Sender<(u64, serde_json::Value)>>,
    writer_ack: Receiver<(u64, Result<(), FrameError>)>,
    writer_done: Receiver<Result<(), std::io::Error>>,
    writer_stop: Arc<AtomicBool>,
    /// 写端取消凭证（停止侧经此取消挂起的同步分片写）。
    writer_cancel: Option<auto_tool_execution::CancelCredential>,
    rx: Receiver<FromReader>,
    reader_stop: Arc<AtomicBool>,
    reader_closed: Arc<AtomicBool>,
    logs: Arc<LogSink>,
    stderr_done: Receiver<Result<(), std::io::Error>>,
    stderr_stop: Arc<AtomicBool>,
    stderr_closed: Arc<AtomicBool>,
    next_id: u64,
    /// 长运行操作终回执之前到达的同 id 事件（Launched/Progress）——
    /// `wait_reply` 驻留，`take_events` 一次取空。
    parked_events: Vec<ToolEvent>,
    /// 控制流回执（同 id InputAck/Cancelled——F-816-T05-R1-02：与工作
    /// 终态分离，不误当 run/build 答案也不静默丢弃）；`take_receipts`。
    parked_receipts: Vec<ServerBody>,
    /// 交互 stdin 泵归属（F-816-T05-R1-03）：run 终态/关闭路径统一
    /// 有界回收（置停+取消挂起读+join），不残留读取者。
    pump: Option<PumpHandle>,
    /// 宿主 describe 缓存（F-816-T05-R2-04 选项协商：携带 op 级执行
    /// 选项的请求发送前按此核对工具广播；None = 尚未查询）。
    descriptor: Option<ToolDescriptor>,
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
        // F-816-R8：凭证创建/复制失败在启动任何自有线程前具名拒绝
        //（宿主进程由 OwnedProcess::drop 终止回收,不泄漏）。
        let stdin = proc.take_stdin().map_err(|e| match e {
            auto_tool_execution::StdinTakeError::Missing => {
                ClientError::InvalidSpec("tool host stdin was not piped".into())
            }
            auto_tool_execution::StdinTakeError::Credential(source) => ClientError::Io {
                stage: "create stdin cancel credential",
                source: Box::new(source),
            },
        })?;
        // 写端取消凭证副本（Windows:停止侧取消挂起的同步写——R5 真停）。
        let writer_cancel = stdin
            .cancel_handle()
            .map(|credential| {
                #[cfg(windows)]
                {
                    Some(credential)
                }
                #[cfg(not(windows))]
                {
                    let _ = credential; // 平台中立签名:Unix 哨兵恒 None
                    None
                }
            })
            .map_err(|source| ClientError::Io {
                stage: "clone stdin cancel credential",
                source: Box::new(source),
            })?;
        let (jobs_tx, jobs_rx) = std::sync::mpsc::channel::<(u64, serde_json::Value)>();
        let (ack_tx, writer_ack) = std::sync::mpsc::channel::<(u64, Result<(), FrameError>)>();
        let (wdone_tx, writer_done) = std::sync::mpsc::channel::<Result<(), std::io::Error>>();
        let writer_stop = Arc::new(AtomicBool::new(false));
        let wflag = Arc::clone(&writer_stop);
        let budget = spec.frame_budget;
        std::thread::spawn(move || {
            let mut stdin = stdin;
            loop {
                let (job_id, job) = match jobs_rx.recv_timeout(PIPE_POLL_SLICE) {
                    Ok(job) => job,
                    Err(RecvTimeoutError::Timeout) => {
                        if wflag.load(Ordering::Relaxed) {
                            break;
                        }
                        continue;
                    }
                    Err(RecvTimeoutError::Disconnected) => break,
                };
                let result = write_frame_chunked(&mut stdin, &job, budget, &wflag);
                // 仅真实写出失败且非停止语境时终结连接;预算/序列化拒绝
                // 发生在任何字节写出之前;stop 取消的拆除属 Stopped 语义
                //（经 done 传播,连接此时已在收尾）。
                let fatal_io =
                    matches!(&result, Err(FrameError::Io(_))) && !wflag.load(Ordering::Relaxed);
                let _ = ack_tx.send((job_id, result));
                if fatal_io {
                    break;
                }
            }
            // 通道关闭（调用方收尾）或致命写失败：drop stdin → 宿主见 EOF。
            drop(stdin);
            let _ = wdone_tx.send(Ok(()));
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
        let logs = Arc::new(LogSink {
            buf: Mutex::new(Vec::new()),
            live: spec.live_log.clone(),
        });
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
            writer_cancel,
            rx,
            reader_stop,
            reader_closed,
            logs,
            stderr_done,
            stderr_stop,
            stderr_closed,
            next_id: 1,
            parked_events: Vec::new(),
            parked_receipts: Vec::new(),
            pump: None,
            descriptor: None,
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
        String::from_utf8_lossy(&self.logs.buf.lock().unwrap()).into_owned()
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

    /// 取缓存的宿主描述（F-816-T05-R2-04：无缓存时先真实 describe 一次
    /// 并缓存——同一连接上后续选项核对零额外往返）。
    fn ensure_descriptor(&mut self) -> Result<ToolDescriptor, ClientError> {
        if let Some(d) = &self.descriptor {
            return Ok(d.clone());
        }
        let d = self.describe()?;
        self.descriptor = Some(d.clone());
        Ok(d)
    }

    /// op 级执行选项协商（F-816-T05-R2-04）：`requested` 中任一选项未被
    /// 宿主能力广播兑现即在**任何帧写出前**具名拒绝——旧 v1 工具对
    /// 新增字段静默忽略（serde 兼容），若放行会丢掉 `output`/`strict`
    /// 等语义继续产生副作用。能力项缺失（操作未提供）同样拒绝。
    fn require_op_options(
        &mut self,
        operation: &'static str,
        requested: &[&str],
    ) -> Result<(), ClientError> {
        if requested.is_empty() {
            return Ok(());
        }
        let descriptor = self.ensure_descriptor()?;
        let capability =
            descriptor
                .capability(operation)
                .ok_or_else(|| ClientError::UnsupportedOption {
                    operation,
                    option: requested[0].to_string(),
                    tool_version: descriptor.tool_version.clone(),
                })?;
        for option in requested {
            if !capability.honors(option) {
                return Err(ClientError::UnsupportedOption {
                    operation,
                    option: option.to_string(),
                    tool_version: descriptor.tool_version.clone(),
                });
            }
        }
        Ok(())
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

    /// 取走本次连接驻留的全部操作事件（Launched/Progress）——终回执之前
    /// 到达的同 id 事件按到达序在此交付（M816-B）。
    pub fn take_events(&mut self) -> Vec<ToolEvent> {
        std::mem::take(&mut self.parked_events)
    }

    /// 取走驻留的控制流回执（InputAck/Cancelled）——输入转发确认与
    /// 取消回执按到达序交付；与工作终态分离（F-816-T05-R1-02）。
    pub fn take_receipts(&mut self) -> Vec<ServerBody> {
        std::mem::take(&mut self.parked_receipts)
    }

    /// generate_ui（M815-B 五操作）：请求工程生成到其声明输出根。有界
    /// 操作——与 describe 同构的请求/回执往返；宿主结构化拒绝作为内层
    /// `Err` 数据返回。
    pub fn generate_ui(
        &mut self,
        request: auto_project_model::GenerationRequest,
    ) -> Result<Result<GenerationReport, ToolError>, ClientError> {
        self.generate_ui_opts(request, &GenerateUiOpts::default())
    }

    /// generate_ui 带执行选项（PLAN-816 T-05 R1 F-01/F-05：旧 `gen
    /// -o/-p/--delivery-root` 与 strict 信号的跨进程承接——op 级可选
    /// 字段，非 I-PROJECT 声明）。
    pub fn generate_ui_opts(
        &mut self,
        request: auto_project_model::GenerationRequest,
        opts: &GenerateUiOpts,
    ) -> Result<Result<GenerationReport, ToolError>, ClientError> {
        // F-816-T05-R2-04：携带的执行选项先经 describe 广播协商——任何
        // 帧写出（=副作用）之前对未广播选项具名拒绝。
        let mut requested: Vec<&str> = Vec::new();
        if opts.output.is_some() {
            requested.push(op_options::OUTPUT);
        }
        if opts.project.is_some() {
            requested.push(op_options::PROJECT);
        }
        if opts.delivery_root.is_some() {
            requested.push(op_options::DELIVERY_ROOT);
        }
        if opts.strict.is_some() {
            requested.push(op_options::STRICT);
        }
        self.require_op_options("generate_ui", &requested)?;
        match self.round_trip_park(
            self.spec.protocol_version,
            ClientBody::GenerateUi {
                request: WireRequest(request),
                output: opts.output.clone(),
                project: opts.project,
                delivery_root: opts.delivery_root.clone(),
                strict: opts.strict,
            },
            false,
        )? {
            Reply::Other(ServerBody::GenerateOk { report }) => {
                // F-816-T05-R2-04 端到端核实：显式 output/delivery_root 的
                // 成功报告必须落在请求落点（R2-02 修复后真实工具保证该
                // 不变量；不符=选项被中间人/宿主静默丢弃，具名失败）。
                if let Some(out) = &opts.output {
                    if std::path::Path::new(&report.output_root) != out.as_path() {
                        return Err(ClientError::OptionIgnored {
                            operation: "generate_ui",
                            option: op_options::OUTPUT,
                            requested: out.display().to_string(),
                            reported: report.output_root.clone(),
                        });
                    }
                }
                if let Some(root) = &opts.delivery_root {
                    if std::path::Path::new(&report.output_root) != root.as_path() {
                        return Err(ClientError::OptionIgnored {
                            operation: "generate_ui",
                            option: op_options::DELIVERY_ROOT,
                            requested: root.display().to_string(),
                            reported: report.output_root.clone(),
                        });
                    }
                }
                Ok(Ok(report))
            }
            Reply::Host(e) => Ok(Err(e)),
            other => Err(ClientError::MalformedReply {
                reason: format!("unexpected reply to generate_ui: {other:?}"),
            }),
        }
    }

    /// build_ui（M815-B 五操作）：生产构建。可取消的长运行操作；事件经
    /// [`ToolClient::take_events`] 交付。
    pub fn build_ui(
        &mut self,
        request: auto_project_model::GenerationRequest,
    ) -> Result<Result<BuildReport, ToolError>, ClientError> {
        self.build_ui_opts(request, None)
    }

    /// build_ui 带 strict 信号（PLAN-816 T-05 R1 F-05）。
    pub fn build_ui_opts(
        &mut self,
        request: auto_project_model::GenerationRequest,
        strict: Option<bool>,
    ) -> Result<Result<BuildReport, ToolError>, ClientError> {
        // F-816-T05-R2-04：显式 strict 信号先协商（旧 v1 工具会静默丢
        // 严格校验继续生成——副作用前具名拒绝）。
        if strict.is_some() {
            self.require_op_options("build_ui", &[op_options::STRICT])?;
        }
        match self.round_trip_park(
            self.spec.protocol_version,
            ClientBody::BuildUi {
                request: WireRequest(request),
                strict,
            },
            false,
        )? {
            Reply::Other(ServerBody::BuildOk { report }) => Ok(Ok(report)),
            Reply::Host(e) => Ok(Err(e)),
            other => Err(ClientError::MalformedReply {
                reason: format!("unexpected reply to build_ui: {other:?}"),
            }),
        }
    }

    /// run_ui（M815-B 五操作）：真实运行链（dev-server/VM 会话/cargo/
    /// jet/ark/...）。可取消、可交互。
    ///
    /// `forward_stdin=true` 时启动**本进程 stdin 泵**：逐行读取宿主进程
    /// 自身 stdin 并以 Input 帧转发到操作的交互子进程（Vite/VM 会话的
    /// `q` 语义）。泵线程读的是本进程 stdin——OS 层不可取消，只能随
    /// 进程生命周期终结（run 返回后 CLI 立即退出）；它不是 owned 设施
    /// 的自有线程，不参与收尾纪律，但持有 jobs 通道克隆——EOF 优雅
    /// 关闭因此依赖 [`ToolClient::shutdown`] 的 writer_stop 置位而非仅
    /// 通道关闭。
    pub fn run_ui(
        &mut self,
        request: auto_project_model::GenerationRequest,
        forward_stdin: bool,
    ) -> Result<Result<RunReport, ToolError>, ClientError> {
        self.run_ui_opts(request, Vec::new(), forward_stdin)
    }

    /// run_ui 带透传参数（PLAN-816 T-05 R1 F-05：`auto run -- <args>`
    /// 旧语义穿进程边界到真实运行链的既有 args 形参）。
    pub fn run_ui_opts(
        &mut self,
        request: auto_project_model::GenerationRequest,
        args: Vec<String>,
        forward_stdin: bool,
    ) -> Result<Result<RunReport, ToolError>, ClientError> {
        // F-816-T05-R2-04：透传参数先协商（旧 v1 工具会静默丢 args 运
        // 行——副作用前具名拒绝）。
        if !args.is_empty() {
            self.require_op_options("run_ui", &[op_options::ARGS])?;
        }
        if forward_stdin {
            self.spawn_stdin_pump()?;
        }
        let result = match self.round_trip_park(
            self.spec.protocol_version,
            ClientBody::RunUi {
                request: WireRequest(request),
                args,
            },
            true,
        ) {
            Ok(reply) => match reply {
                Reply::Other(ServerBody::RunOk { report }) => Ok(Ok(report)),
                Reply::Host(e) => Ok(Err(e)),
                other => Err(ClientError::MalformedReply {
                    reason: format!("unexpected reply to run_ui: {other:?}"),
                }),
            },
            Err(e) => Err(e),
        };
        // F-816-T05-R1-03/R2-03：泵有界回收（置停+重发取消+真停 join）
        // 先于返回；回收失败具名传播——不拿运行成功掩盖残留读者。
        let reclaim = self.reclaim_pump();
        match (result, reclaim) {
            (r, Ok(())) => r,
            (_, Err(e)) => Err(e),
        }
    }

    /// 显式 Input 发送（协议控制消息）：转发原始字节到具名操作的交互
    /// 子进程 stdin；宿主对无交互子的操作具名拒绝（内层 `Err` 数据）。
    pub fn send_input(
        &mut self,
        target_id: u64,
        data: Vec<u8>,
    ) -> Result<Result<(), ToolError>, ClientError> {
        match self.request(ClientBody::Input { target_id, data })? {
            Reply::Other(ServerBody::InputAck { .. }) => Ok(Ok(())),
            Reply::Host(e) => Ok(Err(e)),
            other => Err(ClientError::MalformedReply {
                reason: format!("unexpected reply to input: {other:?}"),
            }),
        }
    }

    /// 本进程 stdin 泵：逐行读取并投递 Input 帧（带操作 id）。返回泵的
    /// 停止标志（run 终回执后置位——阻塞中的读只能随进程退出解除，
    /// 见 [`ToolClient::run_ui`] 的生命周期注记）。
    /// 启动交互 stdin 泵（行级转发为 Input 帧；op id = 即将发出的 run
    /// 请求 id——run_ui_opts 单飞语义下先于请求分配）。泵线程持 jobs
    /// 通道克隆与 std 输入句柄；回收统一走 [`ToolClient::reclaim_pump`]
    /// （F-816-T05-R1-03：有界、可停、不残留）。
    fn spawn_stdin_pump(&mut self) -> Result<(), ClientError> {
        if self.pump.is_some() {
            return Ok(()); // 已在运行（单飞语义下唯一 run 的泵）
        }
        let tx = self
            .jobs_tx
            .clone()
            .ok_or_else(|| ClientError::InvalidSpec("writer already closed".into()))?;
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let version = self.spec.protocol_version;
        let op_id = self.next_id; // 即将发出的 run 请求 id（run_ui_opts 单飞）
        let join = std::thread::Builder::new()
            .name("tool-client-stdin-pump".into())
            .spawn(move || pump_io::pump_loop(op_id, version, tx, flag))
            .map_err(|e| ClientError::Io {
                stage: "spawn stdin pump",
                source: Box::new(e),
            })?;
        self.pump = Some(PumpHandle {
            stop,
            join: Some(join),
        });
        Ok(())
    }

    /// 泵的有界回收（F-816-T05-R1-03/R2-03）：置停 → **切片重发取消直至
    /// 线程真停**（覆盖"取消先于 ReadFile 入核、之后才进入的挂起读"窗口，
    /// 镜像设施 F-R3-02 的凭证重发语义）→ join 收割。宽限尽线程未停 =
    /// 具名 [`ClientError::PumpStuck`]，**句柄保留在槽内**（residue——
    /// 后续 shutdown 再有界重试，不因槽空误判已处理）；join panic = 具名
    /// [`ClientError::Panicked`]。
    fn reclaim_pump(&mut self) -> Result<(), ClientError> {
        let grace = self.spec.shutdown_timeout.min(TEARDOWN_GRACE);
        let Some(mut pump) = self.pump.take() else {
            return Ok(());
        };
        match reclaim_pump_handle(&mut pump, grace) {
            Ok(()) => Ok(()), // 已 join 收割——槽保持空
            Err(e) => {
                // residue：归属不静默丢弃（F-816-T05-R2-03），错误具名传播。
                self.pump = Some(pump);
                Err(e)
            }
        }
    }

    /// 发送一个类型化请求并等待镜像回答（控制请求语义：同 id 的
    /// InputAck/Cancelled 回执即为本请求的终态——不驻留）。
    pub fn request(&mut self, body: ClientBody) -> Result<Reply, ClientError> {
        self.round_trip_park(self.spec.protocol_version, body, false)
    }

    /// round_trip + 驻留策略：`park_controls=true`（工作操作 run）把
    /// 同 id 的控制流回执（InputAck/Cancelled）与事件一并驻留，继续等
    /// 唯一工作终回执（F-816-T05-R1-02：InputAck 不是 run 的答案）；
    /// false 保持控制请求（input/cancel/负控）以回执为终态的旧语义。
    fn round_trip_park(
        &mut self,
        version: u32,
        body: ClientBody,
        park_controls: bool,
    ) -> Result<Reply, ClientError> {
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
        self.send_deadline(&json, started, limit, id, true)?;
        let reply = self.wait_reply(id, limit, started, park_controls)?;
        Ok(typed_reply(reply))
    }

    /// 负控通道 ①：以偏离的协议版本发送类型化请求（宿主须在副作用前
    /// 具名拒绝，流保持可用）。合同验证用，生产路径勿用。
    pub fn request_with_version(
        &mut self,
        version: u32,
        body: ClientBody,
    ) -> Result<Reply, ClientError> {
        self.round_trip_park(version, body, false)
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
        self.send_deadline(&raw, started, limit, id, true)?;
        let reply = self.wait_reply(id, limit, started, false)?;
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
        wait_ack: bool,
    ) -> Result<(), ClientError> {
        let tx = self
            .jobs_tx
            .as_ref()
            .ok_or_else(|| ClientError::InvalidSpec("writer already closed".into()))?;
        tx.send((id, json.clone()))
            .map_err(|_| ClientError::Panicked {
                which: "stdin writer",
            })?;
        if !wait_ack {
            // 交互 pump 的 Input 帧只投递不确认——确认语义只属请求发送方。
            return Ok(());
        }
        // 按 id 过滤的确认：pump 帧的 ack 在此被丢弃，等待只属于本请求。
        let deadline = limit.map(|l| started + l);
        loop {
            let wait = deadline.map(|d| d.saturating_duration_since(Instant::now()));
            let (ack_id, result) = match wait {
                Some(w) => match self.writer_ack.recv_timeout(w) {
                    Ok(pair) => pair,
                    Err(RecvTimeoutError::Timeout) => {
                        return Err(self.abort_for_timeout(id, limit.unwrap_or_default()))
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        return Err(ClientError::Panicked {
                            which: "stdin writer",
                        })
                    }
                },
                None => match self.writer_ack.recv() {
                    Ok(pair) => pair,
                    Err(_) => {
                        return Err(ClientError::Panicked {
                            which: "stdin writer",
                        })
                    }
                },
            };
            if ack_id != id {
                continue;
            }
            return match result {
                Ok(()) => Ok(()),
                Err(e) => {
                    if matches!(e, FrameError::Io(_)) {
                        self.broken = Some(BrokenReason::WriterFailed);
                    }
                    Err(ClientError::SendFrame(e))
                }
            };
        }
    }

    fn wait_reply(
        &mut self,
        id: u64,
        limit: Option<Duration>,
        started: Instant,
        park_controls: bool,
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
                if envelope.id != id {
                    return Err(ClientError::IdMismatch {
                        sent: id,
                        received: envelope.id,
                    });
                }
                // 长运行操作的事件（Progress/Launched）与控制流回执
                // （InputAck/Cancelled——输入转发确认、取消回执）先于
                // 终回执到达且与请求同 id：工作操作等待中一律驻留并继续
                // 等唯一终回执（F-816-T05-R1-02：InputAck 不是 run 的
                // 答案）；控制请求（input/cancel）以回执为终态不驻留。
                match envelope.body {
                    ServerBody::Event { event } => {
                        self.parked_events.push(event);
                        return self.wait_reply(id, limit, started, park_controls);
                    }
                    ServerBody::InputAck { .. } | ServerBody::Cancelled { .. } if park_controls => {
                        self.parked_receipts.push(envelope.body);
                        return self.wait_reply(id, limit, started, park_controls);
                    }
                    _ => {}
                }
                Ok(envelope)
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
        // 泵先行回收（其 jobs 克隆消亡——writer 终止不再被保活）；
        // 回收失败具名进入聚合（取消失败 > 泵残留 > 收尾 drain > Timeout）。
        let pump_err = self.reclaim_pump().err();
        self.writer_stop.store(true, Ordering::Relaxed);
        drop(self.jobs_tx.take());
        if let Some(h) = &self.writer_cancel {
            let _ = h.cancel();
        }
        let cancel_result = self.proc.cancel_within(TEARDOWN_GRACE).map(|_| ());
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
        if let Some(e) = pump_err {
            return e;
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
        // 三条线程全部终结后才传播任一失败（F-816-R3-02；写端超时先
        // 取消阻塞写再短重试——R5 真停）。
        let writer_err = self.wait_writer_done(grace).err();
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

    /// 等待 writer 完成（有界）：等待期间**每轮询切片重发取消**（R7
    /// 窗口闭合——写者任意时刻进入内核的写都会被随后的重发中止）；
    /// 未完成即具名 PipesHeld，不冒充已清理。
    fn wait_writer_done(&mut self, grace: Duration) -> Result<(), ClientError> {
        if self.writer_done_seen {
            return Ok(());
        }
        let deadline = Instant::now() + grace;
        loop {
            let slice = PIPE_POLL_SLICE.min(deadline.saturating_duration_since(Instant::now()));
            match self.writer_done.recv_timeout(slice) {
                Ok(_) => {
                    self.writer_done_seen = true;
                    return Ok(());
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(ClientError::Panicked {
                        which: "stdin writer",
                    })
                }
                Err(RecvTimeoutError::Timeout) => {
                    if let Some(h) = &self.writer_cancel {
                        let _ = h.cancel();
                    }
                    if Instant::now() >= deadline {
                        return Err(ClientError::PipesHeld { after: grace });
                    }
                }
            }
        }
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

    /// 错误路径统一收尾（R5）：停全部线程 + 取消阻塞写 + 有界 drain
    /// 确认；drain 的"线程未停"失败（PipesHeld/Panicked）优先于原错误
    /// 返回（带着活线程返回的事实优先）；其余 drain 结果不覆盖原错误。
    fn cleanup_then_error(&mut self, err: ClientError) -> ClientError {
        self.stop_all();
        if let Some(h) = &self.writer_cancel {
            h.cancel();
        }
        match self.drain_until_done(TEARDOWN_GRACE, false) {
            Err(e @ (ClientError::PipesHeld { .. } | ClientError::Panicked { .. })) => e,
            _ => err,
        }
    }

    /// 优雅关闭：结束 writer（EOF）→ 宽限内等待宿主退出（合同期待码 0）；
    /// 根退出后先终止仍持管道的 owned 后代（F-816-R1-02），三条线程确
    /// 认真停；宽限到期则 cancel 收尸。返回退出分类与完整日志。
    pub fn shutdown(mut self) -> Result<Shutdown, ClientError> {
        let deadline = Instant::now() + self.spec.shutdown_timeout;
        // 0.（F-816-T05-R1-03/R2-03）先有界回收交互泵（置停+重发取消
        //    +真停 join）：泵的 jobs 克隆先消亡；随后 writer 停止位**先于**
        //    任何等待置上——writer 经标志位或通道关闭任一终止，EOF
        //    语义双保险（不再被泵克隆保活拖到宽限尽）。回收失败时句柄
        //    留在槽内（residue），其余关闭步骤照常完成后尾部重试并具名
        //    传播——不以槽空误判已处理。
        let pump_first_err = self.reclaim_pump().err();
        self.writer_stop.store(true, Ordering::Relaxed);
        // 1. 结束 writer（jobs 通道关闭 → 分片写完成 → drop stdin）后,
        //    立即关闭取消凭证副本——EOF 不被副本延迟（R5 探针证实:
        //    副本存活期间宿主收不到 stdin EOF）。
        drop(self.jobs_tx.take());
        let writer_grace = deadline
            .saturating_duration_since(Instant::now())
            .min(TEARDOWN_GRACE);
        if let Err(e) = self.wait_writer_done(writer_grace) {
            return Err(self.cleanup_then_error(e));
        }
        drop(self.writer_cancel.take());
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
                        return Err(self.cleanup_then_error(ClientError::Io {
                            stage: "terminate after root exit",
                            source: Box::new(source),
                        }));
                    }
                }
                status
            }
            Ok(None) => {
                natural_exit = false;
                // 有界取消（R7：终止失败不得无界 wait）。
                match self.proc.cancel_within(TEARDOWN_GRACE) {
                    Ok(status) => status,
                    Err(source) => {
                        return Err(self.cleanup_then_error(ClientError::Io {
                            stage: "cancel after shutdown grace",
                            source: Box::new(source),
                        }));
                    }
                }
            }
            Err(source) => {
                return Err(self.cleanup_then_error(ClientError::Io {
                    stage: "wait for host shutdown",
                    source: Box::new(source),
                }));
            }
        };
        // 3. owned 写端已全关 → reader 的 EOF 判定;停止并确认三条线程。
        // writer 先置停止位再弃通道克隆：交互 pump 可能仍持有一个 jobs
        // 克隆（阻塞读不可弃），EOF 不能只靠通道关闭达成（M816-B）。
        self.writer_stop.store(true, Ordering::Relaxed);
        self.reader_closed.store(true, Ordering::Relaxed);
        self.stderr_closed.store(true, Ordering::Relaxed);
        self.reader_stop.store(true, Ordering::Relaxed);
        self.stderr_stop.store(true, Ordering::Relaxed);
        self.drain_until_done(TEARDOWN_GRACE, natural_exit)?;
        // 4.（F-816-T05-R2-03）泵 residue 终验：步骤 0 未能确认真停的泵
        //    在其余关闭全部完成后重试一次（新一轮重发取消宽限）；仍未
        //    停则具名传播——不宣称干净 Shutdown。
        if self.reclaim_pump().is_err() {
            let e = pump_first_err.unwrap_or(ClientError::PumpStuck {
                after: TEARDOWN_GRACE,
            });
            return Err(e);
        }
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
        other => Reply::Other(other),
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

/// 分片写一帧：16KiB 分片 + 就绪轮询 + 分片间 stop 检查；预算拒绝
/// 发生在写出前。Windows 的 stdin 写端为自建空间探测管道——写永不
/// 阻塞，stop 在分片间必然生效（F-816-R5 写端真停）。
fn write_frame_chunked<W: auto_tool_execution::StoppableWrite>(
    w: &mut W,
    json: &serde_json::Value,
    max_frame_bytes: usize,
    stop: &Arc<AtomicBool>,
) -> Result<(), FrameError> {
    let bytes = frame_payload(json, max_frame_bytes)?;
    let mut written = 0usize;
    while written < bytes.len() {
        let upper = (written + 16 * 1024).min(bytes.len());
        match w.write_stoppable(&bytes[written..upper], stop) {
            Ok(0) => {
                // 停止取消(取消句柄/非阻塞轮询)或写零:停止语境为拆除
                // 语义(由 done/ack 语境归类);否则按写失败具名。
                return Err(FrameError::Io(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "writer stopped mid-frame",
                )));
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
/// stderr 收集槽：缓冲 + 可选活转发（长运行操作的实时人读通道）。
struct LogSink {
    buf: Mutex<Vec<u8>>,
    live: Option<LiveLogFn>,
}

impl LogSink {
    fn push(&self, chunk: &[u8]) {
        self.buf.lock().unwrap().extend_from_slice(chunk);
        if let Some(live) = &self.live {
            live(chunk);
        }
    }
}

fn stderr_session<R>(
    mut r: R,
    sink: &LogSink,
    stop: &AtomicBool,
    owned_closed: &AtomicBool,
) -> Result<(), std::io::Error>
where
    R: Read + PipePoll,
{
    let mut chunk = [0u8; 8192];
    loop {
        if stop.load(Ordering::Relaxed) {
            // F-816-R3-01: 先排空已可读日志再退出——就地写入 sink;
            // R5-01:排空中的读错具名返回，不抹成正常停止。
            let cap = Instant::now() + DRAIN_CAP;
            loop {
                if Instant::now() >= cap {
                    return Ok(());
                }
                if !r.poll_ready(Duration::from_millis(0)) {
                    return Ok(());
                }
                match r.read(&mut chunk) {
                    Ok(0) => return Ok(()),
                    Ok(n) => sink.push(&chunk[..n]),
                    Err(e) => return Err(e),
                }
            }
        }
        if !r.poll_ready(PIPE_POLL_SLICE) {
            if owned_closed.load(Ordering::Relaxed) {
                return Ok(());
            }
            continue;
        }
        match r.read(&mut chunk) {
            Ok(0) => return Ok(()),
            Ok(n) => sink.push(&chunk[..n]),
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

/// stop 排空的结构化结束状态（F-816-R5-01：排空不得把流失败抹成正常
/// 停止）。
#[derive(Debug)]
enum DrainEnd {
    /// 管道空（无更多已到达数据）。
    Empty,
    /// 排空中观察到干净 EOF。
    Eof,
    /// 排空中读错（具名）。
    Io(std::io::Error),
}

/// 排空当前已可读内容到缓冲（F-816-R3-01：stop/收尾=先排空再退出，
/// 已到达数据不得丢弃；R5-01：结束状态结构化返回）。有界：管道空即
/// 止；DRAIN_CAP 防御仍活跃的写入方无限供数。
fn drain_readable_into<R: Read + PipePoll>(r: &mut R, buffer: &mut Vec<u8>) -> DrainEnd {
    let cap = Instant::now() + DRAIN_CAP;
    let mut chunk = [0u8; 8192];
    loop {
        if Instant::now() >= cap {
            return DrainEnd::Empty;
        }
        if !r.poll_ready(Duration::from_millis(0)) {
            return DrainEnd::Empty;
        }
        match r.read(&mut chunk) {
            Ok(0) => return DrainEnd::Eof,
            Ok(n) => buffer.extend_from_slice(&chunk[..n]),
            Err(e) => return DrainEnd::Io(e),
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
            // F-816-R3-01: 先排空 OS 管道中已到达的字节并解析;R5-01:
            // 排空的结束状态按真实终态映射——EOF 夹半帧保留 TruncatedEof
            // 证据（由 natural_exit 消费方判违规），读错具名 Violation。
            // 半帧截断的判据 = "再无数据可到"（排空见 EOF,或 owned 写端
            // 已全关）——R5 注入用 Cursor 型就绪源(读尽后 poll 恒假)以
            // owned_closed 表达流终态。
            match drain_readable_into(r, &mut buffer) {
                DrainEnd::Io(e) => {
                    let _ = tx.send(FromReader::Violation(FrameError::Io(e)));
                    return;
                }
                drain_end => {
                    let live = parse_drain(&mut buffer, &mut pos, tx, max_frame_bytes);
                    let stream_ended =
                        matches!(drain_end, DrainEnd::Eof) || owned_closed.load(Ordering::Relaxed);
                    if live && stream_ended {
                        if buffer.len() > pos {
                            let _ = tx.send(FromReader::TruncatedEof {
                                trailing: buffer.len() - pos,
                            });
                        } else {
                            let _ = tx.send(FromReader::Eof);
                        }
                    }
                    return;
                }
            }
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
        // R5-01：排空观察到干净 EOF——诚实发送 Eof 终态（再 Done）。
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Eof));
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Done));
    }

    /// F-816-R5-01 矩阵——stop 排空遇半帧+EOF：TruncatedEof 证据必须
    /// 保留（长度 64 前缀+不完整载荷），由 natural_exit 消费方判违规。
    #[test]
    fn stop_drain_preserves_truncated_eof_evidence() {
        let (tx, rx) = std::sync::mpsc::channel::<FromReader>();
        let (stop, closed) = flags();
        stop.store(true, Ordering::Relaxed);
        closed.store(true, Ordering::Relaxed);
        let mut partial = (64u32).to_be_bytes().to_vec();
        partial.extend_from_slice(b"{\"id\": 1"); // 不足 64 字节载荷
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
            other => panic!("expected TruncatedEof preserved, got {other:?}"),
        }
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Done));
    }

    /// F-816-R5-01 矩阵——stop 排空遇读错：具名 Violation(Io)，不冒充
    /// 正常停止。
    #[test]
    fn stop_drain_read_error_is_named_violation() {
        let (tx, rx) = std::sync::mpsc::channel::<FromReader>();
        let (stop, closed) = flags();
        stop.store(true, Ordering::Relaxed);
        let mut reader = Chunked {
            idle_ready: true,
            chunks: vec![b"partial-bytes".to_vec()],
            tail: Tail::Err("kernel read gone during stop-drain"),
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
                assert!(e.to_string().contains("stop-drain"), "{e}")
            }
            other => panic!("expected named Io violation, got {other:?}"),
        }
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Done));
    }

    // ---- F-816-T05-R2-03：泵回收真停 / 残留归属 / 具名传播 ----

    /// 宽限内不观察停止位的"挂起读"线程：回收具名 `PumpStuck`（不静默
    /// 返回）、JoinHandle 保留在句柄内（residue 归属不丢）；线程随后
    /// 结束（审方早取消→释放时序的晚侧）时重试回收真停+join 收割。
    /// 对应审方 pump-race 探针的 cancel-before-read 态：原缺陷=回收
    /// 静默返回 () 且槽已空、线程残活；修复后=具名 Err+归属保留，释放
    /// 后可收敛。
    #[test]
    fn pump_reclaim_failure_is_named_and_ownership_kept() {
        let gate = Arc::new(AtomicBool::new(false));
        let g = gate.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let join = std::thread::Builder::new()
            .spawn(move || {
                // 模拟晚入核的挂起读：既不观察 stop，也在 gate 置位前
                // 不结束（重发取消对"尚未入核"的读无效的同构替身）。
                while !g.load(Ordering::SeqCst) {
                    std::thread::sleep(Duration::from_millis(5));
                }
            })
            .unwrap();
        let mut pump = PumpHandle {
            stop,
            join: Some(join),
        };
        let err = reclaim_pump_handle(&mut pump, Duration::from_millis(120)).unwrap_err();
        assert!(
            matches!(err, ClientError::PumpStuck { .. }),
            "named PumpStuck, got {err:?}"
        );
        assert!(
            pump.join.is_some(),
            "residue ownership kept — join handle not silently dropped"
        );
        // 释放（审方探针的 PUMP_RELEASE）：线程退出后重试回收必须在
        // 宽限内确认真停并 join 收割（残留收敛，不再依赖进程退出）。
        gate.store(true, Ordering::SeqCst);
        let retry_deadline = std::time::Instant::now() + Duration::from_secs(2);
        loop {
            if reclaim_pump_handle(&mut pump, Duration::from_secs(2)).is_ok() {
                break;
            }
            assert!(
                std::time::Instant::now() < retry_deadline,
                "released pump must reclaim within grace"
            );
        }
        assert!(pump.join.is_none(), "join reaped on success");
    }

    /// 泵线程 panic：回收以具名 `Panicked` 传播，不吞掉。
    #[test]
    fn pump_reclaim_panicked_thread_is_named() {
        let stop = Arc::new(AtomicBool::new(false));
        let join = std::thread::Builder::new()
            .spawn(|| panic!("pump boom"))
            .unwrap();
        let mut pump = PumpHandle {
            stop,
            join: Some(join),
        };
        let err = reclaim_pump_handle(&mut pump, Duration::from_secs(2)).unwrap_err();
        assert!(
            matches!(
                err,
                ClientError::Panicked {
                    which: "stdin pump"
                }
            ),
            "named Panicked, got {err:?}"
        );
        assert!(pump.join.is_none(), "panicked handle joined (not residue)");
    }

    /// F-816-R5-01（stderr 面）——stop 排空遇读错：具名 Err，不抹成
    /// Ok(())。
    #[test]
    fn stderr_stop_drain_read_error_is_named() {
        let (stop, closed) = flags();
        stop.store(true, Ordering::Relaxed);
        let sink = LogSink {
            buf: Mutex::new(Vec::new()),
            live: None,
        };
        let reader = Chunked {
            idle_ready: true,
            chunks: vec![b"partial-log".to_vec()],
            tail: Tail::Err("stderr kernel gone"),
        };
        let end = stderr_session(reader, &sink, &stop, &closed);
        match end {
            Err(e) => assert!(e.to_string().contains("stderr kernel gone")),
            Ok(()) => panic!("expected named error from stop-drain, got Ok"),
        }
        let logs = String::from_utf8(sink.buf.into_inner().unwrap()).unwrap();
        assert!(logs.contains("partial-log"), "bytes before the error kept");
    }

    /// F-816-R3-01（stderr 面）：stop 先排空已到达日志——预置日志行+
    /// 先行置 stop，sink 必须完整包含（R3 实机间歇缺字根因的确定性反例）。
    #[test]
    fn stderr_stop_drains_preset_bytes() {
        let (stop, closed) = flags();
        stop.store(true, Ordering::Relaxed);
        let sink = LogSink {
            buf: Mutex::new(Vec::new()),
            live: None,
        };
        let reader = Chunked {
            idle_ready: true,
            chunks: vec![b"accepted request id=1 op=describe
"
            .to_vec()],
            tail: Tail::Eof,
        };
        let end = stderr_session(reader, &sink, &stop, &closed);
        assert!(end.is_ok());
        let logs = String::from_utf8(sink.buf.into_inner().unwrap()).unwrap();
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

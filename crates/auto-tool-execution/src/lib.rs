//! PLAN-816 T-02:统一 owned 子进程执行设施。
//!
//! 一份设施拥有:argv 数组、显式 cwd/env、stdin/stdout/stderr 策略
//! (继承/管道/null)、退出/错误具名分类、deadline(毫秒)、cancel 与
//! owned 收尸、捕获 reader 线程真终止、子进程树退出(Windows Job
//! Object / Unix 进程组)。分散在 `crates/auto/src/main.rs`
//! (run_command_live)、`auto-man/src/pkg.rs`(run_script_live/
//! ChildGuard)、`git.rs`、`http_service.rs`(serve_rust)的启动/等待
//! 语义在 T-03/T-05/T-06 迁移消费本设施,不建第二套并存 runner。
//!
//! 边界(PLAN-816 §5.2,合同基线 §1 冻结):
//! - 只终止本次拥有的进程(树);legacy run 链的 `kill_process_on_port`
//!   端口清场不在此处,亦不得扩散到新工具/fixture 或 `auto service`。
//! - stdout/stderr 捕获无默认截断预算(与语言 sys.exec 的 64KB 截断
//!   合同相反;截断属调用方策略)。
//! - cancel/超时后在子进程被观察停止(收尸)后才返回;kill 发出不算已停。
//! - deadline 覆盖整次执行:写入(stdin)、等待与资源收尾共用同一绝对
//!   截止(F-816-R1-01);根进程退出后仍持管道的 owned 后代先终止、
//!   reader 有界 join 后才返回(F-816-R1-02)。
//! - [`StdioPolicy::Inherit`] + [`TreeScope::ProcessTree`] 在 Unix 下使
//!   子进程脱离终端前台组(Ctrl+C 不再直达),交互式命令保持
//!   [`TreeScope::DirectChildOnly`]。
//! - reader 的读错/panic 与 writer/终止失败具名传播,不以成功码或空
//!   输出掩盖(F-816-R1-06)。

use std::ffi::OsString;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[cfg(unix)]
mod unix_pg;
#[cfg(windows)]
mod win_job;

/// 每流 stdio 策略。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StdioPolicy {
    /// 继承调用方 stdio(交互命令;现行 pkg.rs 语义)。
    #[default]
    Inherit,
    /// 管道:stdout/stderr=捕获或调用方读取;stdin=调用方写入
    /// (spawn 后经 [`OwnedProcess::stdin`]/[`OwnedProcess::stdout`] 取)。
    Piped,
    /// 丢弃。
    Null,
}

impl StdioPolicy {
    fn to_stdio(self) -> Stdio {
        match self {
            StdioPolicy::Inherit => Stdio::inherit(),
            StdioPolicy::Piped => Stdio::piped(),
            StdioPolicy::Null => Stdio::null(),
        }
    }
}

/// owned 终止的树范围。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TreeScope {
    /// 仅直接子进程(std `Child::kill` 语义;终端信号行为与现行一致)。
    #[default]
    DirectChildOnly,
    /// owned 树:Windows=Job Object(KILL_ON_JOB_CLOSE +
    /// TerminateJobObject);Unix=子进程新进程组(取消 = killpg)。
    /// 孙进程一并退出。
    ProcessTree,
}

/// 一次执行的完整请求。
#[derive(Clone, Debug, Default)]
pub struct ExecutionSpec {
    /// argv 首元素为程序,其余为参数;零 shell 解释(空 argv 报
    /// [`ExecutionError::InvalidSpec`])。
    pub argv: Vec<OsString>,
    /// 工作目录(None = 继承调用方)。
    pub cwd: Option<PathBuf>,
    /// 在继承环境之上追加/覆盖(显式对;不提供清空基线——需要环境
    /// 隔离的上层自管)。
    pub env: Vec<(OsString, OsString)>,
    pub stdin: StdioPolicy,
    pub stdout: StdioPolicy,
    pub stderr: StdioPolicy,
    /// run() 便捷写入:Some 要求 stdin == Piped(写入并关闭后等待)。
    pub stdin_data: Option<Vec<u8>>,
    /// 运行时限;到期终止 owned 进程(树)并收尸,分类
    /// [`ExitKind::Timeout`]。spawn() 不消费本字段(由调用方用
    /// [`OwnedProcess::wait_deadline`] 自管)。
    pub deadline: Option<Duration>,
    pub tree_scope: TreeScope,
    /// Windows 专属:经 `cmd /C <argv...>` 解析 .cmd/.bat shim
    /// (镜像现行 run_command_live/pkg.rs;参数原样拼接,/C 后引号
    /// 规则归 cmd.exe)。非 Windows 配置 true 报 InvalidSpec。
    pub windows_cmd_shim: bool,
}

impl ExecutionSpec {
    /// 便捷构造:argv 首元素为程序,其余为参数。
    pub fn new<I, S>(argv: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        ExecutionSpec {
            argv: argv.into_iter().map(Into::into).collect(),
            stdin: StdioPolicy::Null,
            stdout: StdioPolicy::Piped,
            stderr: StdioPolicy::Piped,
            ..Default::default()
        }
    }
}

/// 退出分类(具名;Windows 被强杀的进程 code 由终止方给定,故取消/超时
/// 由调用上下文分类,不误读 code)。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExitKind {
    /// 退出码 0。
    Success,
    /// 正常退出非零(cmd /C shim 透传码同此)。
    Failure(i32),
    /// deadline 到期;owned 进程(树)已被终止并收尸,`after` 为触发时
    /// 已运行时长。
    Timeout { after: Duration },
    /// 调用方 cancel;已终止并收尸。
    Cancelled,
    /// 未经本设施终止而以信号退出(Unix;`signal`=-1 表示平台未提供
    /// 信号编号的兜底)。
    Terminated { signal: i32 },
}

impl ExitKind {
    /// 从原始 [`ExitStatus`] 分类(无调用方上下文;取消/超时分类由
    /// `run`/调用方附加)。
    pub fn from_status(status: &ExitStatus) -> ExitKind {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            if let Some(sig) = status.signal() {
                return ExitKind::Terminated { signal: sig };
            }
        }
        match status.code() {
            Some(0) => ExitKind::Success,
            Some(c) => ExitKind::Failure(c),
            None => ExitKind::Terminated { signal: -1 },
        }
    }
}

/// 一次性执行结果。
#[derive(Clone, Debug)]
pub struct ExecutionResult {
    pub kind: ExitKind,
    /// stdout 捕获(仅 stdout 策略为 Piped 时)。
    pub stdout: Option<Vec<u8>>,
    /// stderr 捕获(仅 stderr 策略为 Piped 时)。
    pub stderr: Option<Vec<u8>>,
    pub elapsed: Duration,
}

impl ExecutionResult {
    pub fn stdout_lossy(&self) -> Option<std::borrow::Cow<'_, str>> {
        self.stdout.as_ref().map(|b| String::from_utf8_lossy(b))
    }

    pub fn stderr_lossy(&self) -> Option<std::borrow::Cow<'_, str>> {
        self.stderr.as_ref().map(|b| String::from_utf8_lossy(b))
    }
}

/// 具名执行错误(派生失败含程序名,便于诊断)。
#[derive(Debug)]
pub enum ExecutionError {
    Spawn {
        program: OsString,
        source: io::Error,
    },
    InvalidSpec(String),
    Io {
        stage: &'static str,
        source: io::Error,
    },
    /// 捕获 reader 以 I/O 错误(而非干净 EOF)结束——捕获内容可能已被
    /// 截断,不以成功码掩盖(F-816-R1-06)。
    ReaderIo {
        stream: &'static str,
        source: io::Error,
    },
    /// 捕获 reader 线程 panic(join 失败),同上不掩盖。
    ReaderPanicked {
        stream: &'static str,
    },
    /// 根进程已退出且 owned 树已终止,但管道写端仍被**非本次拥有**的
    /// 进程持有:reader 在宽限内收不到 EOF,按具名失败返回而非无限
    /// 等待/谎报成功(F-816-R1-02)。捕获内容不可信,已丢弃。
    PipesHeld {
        after: Duration,
    },
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionError::Spawn { program, source } => write!(
                f,
                "spawn failed for `{}`: {}",
                program.to_string_lossy(),
                source
            ),
            ExecutionError::InvalidSpec(reason) => write!(f, "invalid execution spec: {reason}"),
            ExecutionError::Io { stage, source } => write!(f, "{stage}: {source}"),
            ExecutionError::ReaderIo { stream, source } => write!(
                f,
                "capture reader for {stream} ended with an I/O error (output may be \
                 truncated): {source}"
            ),
            ExecutionError::ReaderPanicked { stream } => write!(
                f,
                "capture reader for {stream} panicked; output may be truncated"
            ),
            ExecutionError::PipesHeld { after } => write!(
                f,
                "output pipes still held by processes outside this execution after \
                 {after:?}; captured output was dropped"
            ),
        }
    }
}

impl std::error::Error for ExecutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ExecutionError::Spawn { source, .. }
            | ExecutionError::Io { source, .. }
            | ExecutionError::ReaderIo { source, .. } => Some(source),
            ExecutionError::InvalidSpec(_)
            | ExecutionError::ReaderPanicked { .. }
            | ExecutionError::PipesHeld { .. } => None,
        }
    }
}

fn validate(spec: &ExecutionSpec) -> Result<(), ExecutionError> {
    if spec.argv.is_empty() {
        return Err(ExecutionError::InvalidSpec("argv must not be empty".into()));
    }
    if spec.stdin_data.is_some() && spec.stdin != StdioPolicy::Piped {
        return Err(ExecutionError::InvalidSpec(
            "stdin_data requires stdin: Piped".into(),
        ));
    }
    #[cfg(not(windows))]
    if spec.windows_cmd_shim {
        return Err(ExecutionError::InvalidSpec(
            "windows_cmd_shim is Windows-only".into(),
        ));
    }
    Ok(())
}

fn build_command(spec: &ExecutionSpec) -> Command {
    let mut cmd;
    #[cfg(windows)]
    if spec.windows_cmd_shim {
        cmd = Command::new("cmd");
        cmd.arg("/C");
        cmd.args(&spec.argv);
    } else {
        cmd = Command::new(&spec.argv[0]);
        cmd.args(&spec.argv[1..]);
    }
    #[cfg(not(windows))]
    {
        cmd = Command::new(&spec.argv[0]);
        cmd.args(&spec.argv[1..]);
    }
    if let Some(cwd) = &spec.cwd {
        cmd.current_dir(cwd);
    }
    for (k, v) in &spec.env {
        cmd.env(k, v);
    }
    cmd.stdin(spec.stdin.to_stdio());
    cmd.stdout(spec.stdout.to_stdio());
    cmd.stderr(spec.stderr.to_stdio());
    #[cfg(unix)]
    if spec.tree_scope == TreeScope::ProcessTree {
        use std::os::unix::process::CommandExt;
        // 子进程先入新进程组再 exec,消除父侧 setpgid 与 exec 的竞态;
        // setpgid 属 async-signal-safe,pre_exec 内合法。
        unsafe {
            cmd.pre_exec(unix_pg::child_setpgid);
        }
    }
    cmd
}

/// 启动一个 owned 长运行进程。管道句柄经 [`OwnedProcess`] 的
/// stdin/stdout/stderr 访问;寿命(deadline/cancel)由调用方用
/// [`OwnedProcess::wait_deadline`]/[`OwnedProcess::cancel`] 自管。
pub fn spawn(spec: &ExecutionSpec) -> Result<OwnedProcess, ExecutionError> {
    validate(spec)?;
    let mut cmd = build_command(spec);
    #[cfg(windows)]
    if spec.tree_scope == TreeScope::ProcessTree {
        // 挂起启动(F-816-R1-04):任何用户代码——包括派生后代——只在
        // Job 归属建立并恢复主线程之后运行,后代不可能逃出 owned Job。
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(win_job::suspended_creation_flags());
    }
    let child = cmd.spawn().map_err(|source| ExecutionError::Spawn {
        program: spec.argv[0].clone(),
        source,
    })?;
    let mut proc = OwnedProcess {
        child,
        #[cfg(windows)]
        job: None,
        tree_scope: spec.tree_scope,
        reaped: false,
    };
    if spec.tree_scope == TreeScope::ProcessTree {
        #[cfg(windows)]
        {
            match win_job::assign_suspended_and_resume(&proc.child) {
                Ok(job) => proc.job = Some(job),
                Err(source) => {
                    // 根仍处挂起/未恢复状态,不可能已有后代;终止+收尸即
                    // 完成全部清理。
                    let _ = proc.child.kill();
                    let _ = proc.child.wait();
                    return Err(ExecutionError::Io {
                        stage: "assign job object",
                        source,
                    });
                }
            }
        }
    }
    Ok(proc)
}

/// reader 结束方式（具名；干净 EOF、I/O 错误与设施主动停止分开——
/// F-816-R1-06/R2-06）。
#[derive(Debug)]
enum ReaderEnd {
    Eof,
    Io(io::Error),
    /// 设施主动停止（stop 标志）：管道写端仍被非拥有方持有时收尾自有
    /// reader 的手段；捕获字节截至停止点，语义由调用方语境决定。
    Stopped,
}

/// 捕获流的产出：字节 + 结束方式。线程 panic 时通道断开、无消息。
struct Capture {
    bytes: Vec<u8>,
    end: ReaderEnd,
}

/// 轮询切片：stop/owned_closed 的最大生效延迟（公开供客户端复用同一
/// 停止性节奏）。
pub const PIPE_POLL_SLICE: Duration = Duration::from_millis(50);

#[cfg(windows)]
mod pipe_wait {
    use std::ffi::c_void;
    use std::io;
    use std::os::windows::io::AsRawHandle;

    extern "system" {
        fn PeekNamedPipe(
            hNamedPipe: *mut c_void,
            lpBuffer: *mut c_void,
            nBufferSize: u32,
            lpBytesRead: *mut u32,
            lpTotalBytesAvail: *mut u32,
            lpBytesLeftThisMessage: *mut u32,
        ) -> i32;
    }

    /// 读端就绪 = 管道内已有数据(PeekNamedPipe,零阻塞)。同步匿名管道
    /// 句柄对 WaitForSingleObject 恒 signaled,不可用作读就绪(F-816-R2-06
    /// 实测),EOF 判定改由 `owned_closed` 标志承担(owned 写端全关)。
    pub(super) fn read_ready(handle: *mut c_void) -> io::Result<bool> {
        let mut avail: u32 = 0;
        let ok = unsafe {
            PeekNamedPipe(
                handle,
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                &mut avail,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(avail > 0)
        }
    }

    pub(super) fn as_handle<T: AsRawHandle>(t: &T) -> *mut c_void {
        t.as_raw_handle()
    }
}

#[cfg(unix)]
mod pipe_wait {
    use std::os::unix::io::AsRawFd;
    use std::time::Duration;

    #[repr(C)]
    struct PollFd {
        fd: i32,
        events: i16,
        revents: i16,
    }

    extern "C" {
        fn poll(fds: *mut PollFd, nfds: u64, timeout: i32) -> i32;
    }

    const POLLIN: i16 = 0x001;
    const POLLOUT: i16 = 0x004;

    pub(super) fn fd_ready(fd: i32, events: i16, timeout: Duration) -> bool {
        let mut fds = PollFd {
            fd,
            events,
            revents: 0,
        };
        let ms = timeout.as_millis().min(i32::MAX as u128) as i32;
        unsafe { poll(&mut fds, 1, ms) > 0 }
    }

    pub(super) fn as_fd<T: AsRawFd>(t: &T) -> i32 {
        t.as_raw_fd()
    }
}

/// 管道端"就绪"轮询(F-816-R2-06):使自有 reader/writer 线程**可被设施
/// 停止**——每个切片检查 stop/owned_closed 标志,收尾路径确认真停后才
/// 返回(完成通知不等于资源停止,也不带活线程返回)。
pub trait PipePoll {
    /// 读端:管道内已有数据(读不会长期阻塞);写端:可尝试写入。
    /// `false` = 空闲,调用方检查停止/EOF 标志后继续轮询。
    fn poll_ready(&self, slice: Duration) -> bool;
}

#[cfg(windows)]
impl<T: std::os::windows::io::AsRawHandle> PipePoll for T {
    fn poll_ready(&self, _slice: Duration) -> bool {
        // 读端:peek 有数据才就绪(EOF 由 owned_closed 判定);写端:
        // 同步句柄无就绪查询,恒"可尝试"(阻塞写由 owned 终止解除;
        // 非拥有方持读端的极端情形由 stop_and_join 具名 PipesHeld)。
        pipe_wait::read_ready(pipe_wait::as_handle(self)).unwrap_or(true)
    }
}

#[cfg(unix)]
impl PipePoll for std::process::ChildStdout {
    fn poll_ready(&self, slice: Duration) -> bool {
        pipe_wait::fd_ready(pipe_wait::as_fd(self), 0x001, slice)
    }
}

#[cfg(unix)]
impl PipePoll for std::process::ChildStderr {
    fn poll_ready(&self, slice: Duration) -> bool {
        pipe_wait::fd_ready(pipe_wait::as_fd(self), 0x001, slice)
    }
}

#[cfg(unix)]
impl PipePoll for std::process::ChildStdin {
    fn poll_ready(&self, slice: Duration) -> bool {
        pipe_wait::fd_ready(pipe_wait::as_fd(self), 0x004, slice)
    }
}

#[cfg(unix)]
impl PipePoll for std::io::PipeReader {
    fn poll_ready(&self, slice: Duration) -> bool {
        pipe_wait::fd_ready(pipe_wait::as_fd(self), 0x001, slice)
    }
}

#[cfg(unix)]
impl PipePoll for std::io::PipeWriter {
    fn poll_ready(&self, slice: Duration) -> bool {
        pipe_wait::fd_ready(pipe_wait::as_fd(self), 0x004, slice)
    }
}

/// 排空当前已可读的内容（F-816-R3-01：stop/收尾语义=先排空再退出）。
/// 有界：管道空即止；DRAIN_CAP 防御仍活跃的写入方无限供数（stop 仅在
/// owned 树终止后的收尾语境使用,正常情况下数据有限）。
fn drain_readable<R: io::Read + PipePoll>(r: &mut R, bytes: &mut Vec<u8>, chunk: &mut [u8]) {
    let cap = Instant::now() + DRAIN_CAP;
    while Instant::now() < cap {
        if !r.poll_ready(Duration::from_millis(0)) {
            break;
        }
        match r.read(chunk) {
            Ok(0) => break,
            Ok(n) => bytes.extend_from_slice(&chunk[..n]),
            Err(_) => break,
        }
    }
}

/// 排空上限（防御性;owned 树已终止时数据有限,远用不满）。
const DRAIN_CAP: Duration = Duration::from_millis(500);

/// 一条捕获 reader 线程的控制器。
///
/// 两个停止性标志（F-816-R2-06）：
/// - `stop`：设施收尾（超时/取消/关闭）——线程以 `Stopped` 结束，
///   字节截至停止点；
/// - `owned_closed`：owned 写端已全部关闭——Windows 同步管道无 EOF
///   轮询，peek 零数据 + 本标志即判 `Eof`（Unix 由 poll 的 HUP 覆盖，
///   标志仅作加速）。
struct CaptureWorker {
    stream: &'static str,
    stop: Arc<AtomicBool>,
    owned_closed: Arc<AtomicBool>,
    done: Receiver<Capture>,
    join: Option<JoinHandle<()>>,
}

impl CaptureWorker {
    fn spawn<R>(stream: &'static str, r: R) -> Self
    where
        R: io::Read + PipePoll + Send + 'static,
    {
        let (tx, done) = std::sync::mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let owned_closed = Arc::new(AtomicBool::new(false));
        let (flag, closed) = (Arc::clone(&stop), Arc::clone(&owned_closed));
        let join = std::thread::spawn(move || {
            let mut r = r;
            let mut bytes = Vec::new();
            let mut end = ReaderEnd::Eof;
            let mut chunk = [0u8; 16 * 1024];
            loop {
                if flag.load(Ordering::Relaxed) {
                    // F-816-R3-01: stop 不丢弃已到达数据——先排空可读
                    // 内容再以 Stopped 结束(捕获字节截至真正排空点)。
                    drain_readable(&mut r, &mut bytes, &mut chunk);
                    end = ReaderEnd::Stopped;
                    break;
                }
                if !r.poll_ready(PIPE_POLL_SLICE) {
                    if closed.load(Ordering::Relaxed) {
                        // owned 写端已全关且无数据:EOF。
                        break;
                    }
                    continue;
                }
                match r.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => bytes.extend_from_slice(&chunk[..n]),
                    Err(e)
                        if e.kind() == io::ErrorKind::WouldBlock
                            || e.kind() == io::ErrorKind::Interrupted =>
                    {
                        continue
                    }
                    Err(e) => {
                        end = ReaderEnd::Io(e);
                        break;
                    }
                }
            }
            let _ = tx.send(Capture { bytes, end });
        });
        Self {
            stream,
            stop,
            owned_closed,
            done,
            join: Some(join),
        }
    }

    /// 请求停止（不等待）。
    fn request_stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    /// 标记 owned 写端已全部关闭（EOF 判定依据之一）。
    fn mark_owned_closed(&self) {
        self.owned_closed.store(true, Ordering::Relaxed);
    }

    /// 等待自然完成（EOF/Io/panic），不请求停止；None = 宽限内未完成
    ///（如管道被非拥有方持有）。`Some(Err(()))` = 线程 panic。
    fn wait(&self, grace: Duration) -> Option<Result<Capture, ()>> {
        match self.done.recv_timeout(grace) {
            Ok(cap) => Some(Ok(cap)),
            Err(RecvTimeoutError::Disconnected) => Some(Err(())),
            Err(RecvTimeoutError::Timeout) => None,
        }
    }

    /// 请求停止并等待线程**真停**。join 只在收到完成消息（或确认断开）
    /// 后进行——对一个仍阻塞在读里的线程 join 会永久悬挂；宽限内未停
    /// 即 [`ExecutionError::PipesHeld`]（具名而不谎报、不悬挂）。
    fn stop_and_join(&mut self, grace: Duration) -> Result<Capture, ExecutionError> {
        self.request_stop();
        let outcome = self.done.recv_timeout(grace);
        if !matches!(outcome, Err(RecvTimeoutError::Timeout)) {
            let _ = self.join.take().map(|h| h.join());
        }
        match outcome {
            Ok(cap) => Ok(cap),
            Err(RecvTimeoutError::Disconnected) => Err(ExecutionError::ReaderPanicked {
                stream: self.stream,
            }),
            Err(RecvTimeoutError::Timeout) => Err(ExecutionError::PipesHeld { after: grace }),
        }
    }
}

/// 正常路径的捕获收集：EOF 才算成功；I/O 错误/panic 具名传播；宽限内
/// 无 EOF（管道被**非本次拥有**的进程持有）→ 先停止并 join 自有线程
/// 再返回 [`ExecutionError::PipesHeld`]——不带活线程返回（R2-06）。
fn finish_capture(
    worker: &mut Option<CaptureWorker>,
    grace: Duration,
) -> Result<Option<Vec<u8>>, ExecutionError> {
    let Some(w) = worker else {
        return Ok(None);
    };
    match w.wait(grace) {
        Some(Ok(cap)) => match cap.end {
            ReaderEnd::Eof => Ok(Some(cap.bytes)),
            ReaderEnd::Io(source) => Err(ExecutionError::ReaderIo {
                stream: w.stream,
                source,
            }),
            ReaderEnd::Stopped => Err(ExecutionError::PipesHeld { after: grace }),
        },
        Some(Err(())) => {
            w.stop_and_join(TEARDOWN_GRACE)?;
            Err(ExecutionError::ReaderPanicked { stream: w.stream })
        }
        None => {
            w.stop_and_join(TEARDOWN_GRACE)?;
            Err(ExecutionError::PipesHeld { after: grace })
        }
    }
}

/// stdin 写入结果（具名）：全部写完、写错或被设施停止（部分写入）。
#[derive(Debug)]
struct WriterOutcome {
    #[cfg_attr(not(test), allow(dead_code))]
    written: usize,
    end: WriterEnd,
}

#[derive(Debug)]
enum WriterEnd {
    Done,
    Io(io::Error),
    Stopped,
}

/// stdin 写入线程控制器：分块写入 + 就绪轮询，可被设施停止（对端不读
/// 且管道写端读侧被非拥有方持有时，deadline 收尾不悬挂，F-816-R1/R2）。
/// Windows 同步写端无就绪查询：阻塞分片写由 owned 终止解除；非拥有方
/// 持读端的极端情形由 stop_and_join 具名 PipesHeld（不谎报、不悬挂）。
struct WriterWorker {
    stop: Arc<AtomicBool>,
    done: Receiver<WriterOutcome>,
    join: Option<JoinHandle<()>>,
}

/// 单次阻塞写的分片上限：写阻塞窗口受此约束（对端 draining 时每个
/// 分片很快完成）。
const WRITE_CHUNK: usize = 16 * 1024;

impl WriterWorker {
    fn spawn<W>(stdin: W, data: Vec<u8>) -> Self
    where
        W: Write + PipePoll + Send + 'static,
    {
        let (tx, done) = std::sync::mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let join = std::thread::spawn(move || {
            let mut stdin = stdin;
            let mut written = 0usize;
            let mut end = WriterEnd::Done;
            loop {
                if written == data.len() {
                    break;
                }
                if flag.load(Ordering::Relaxed) {
                    end = WriterEnd::Stopped;
                    break;
                }
                let upper = (written + WRITE_CHUNK).min(data.len());
                match stdin.write(&data[written..upper]) {
                    Ok(0) => {
                        end = WriterEnd::Io(io::Error::from(io::ErrorKind::WriteZero));
                        break;
                    }
                    Ok(n) => written += n,
                    Err(e)
                        if e.kind() == io::ErrorKind::WouldBlock
                            || e.kind() == io::ErrorKind::Interrupted =>
                    {
                        continue
                    }
                    Err(e) => {
                        end = WriterEnd::Io(e);
                        break;
                    }
                }
            }
            // drop(stdin) 关闭管道，子进程 stdin 见 EOF。
            drop(stdin);
            let _ = tx.send(WriterOutcome { written, end });
        });
        Self {
            stop,
            done,
            join: Some(join),
        }
    }

    fn request_stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    fn wait(&self, grace: Duration) -> Option<Result<WriterOutcome, ()>> {
        match self.done.recv_timeout(grace) {
            Ok(out) => Some(Ok(out)),
            Err(RecvTimeoutError::Disconnected) => Some(Err(())),
            Err(RecvTimeoutError::Timeout) => None,
        }
    }

    fn stop_and_join(&mut self, grace: Duration) -> Result<WriterOutcome, ExecutionError> {
        self.request_stop();
        let outcome = self.done.recv_timeout(grace);
        if !matches!(outcome, Err(RecvTimeoutError::Timeout)) {
            let _ = self.join.take().map(|h| h.join());
        }
        match outcome {
            Ok(out) => Ok(out),
            Err(RecvTimeoutError::Disconnected) => Err(ExecutionError::ReaderPanicked {
                stream: "stdin writer",
            }),
            Err(RecvTimeoutError::Timeout) => Err(ExecutionError::PipesHeld { after: grace }),
        }
    }
}

/// 正常路径的 writer 收集：全部写完才算成功；写错具名传播；宽限内
/// 未完成 → 停止+join 后 [`ExecutionError::PipesHeld`]。
fn finish_writer(worker: &mut Option<WriterWorker>, grace: Duration) -> Result<(), ExecutionError> {
    let Some(w) = worker else {
        return Ok(());
    };
    match w.wait(grace) {
        Some(Ok(out)) => match out.end {
            WriterEnd::Done => Ok(()),
            WriterEnd::Io(source) => Err(ExecutionError::Io {
                stage: "write stdin",
                source,
            }),
            WriterEnd::Stopped => Err(ExecutionError::PipesHeld { after: grace }),
        },
        Some(Err(())) => {
            w.stop_and_join(TEARDOWN_GRACE)?;
            Err(ExecutionError::ReaderPanicked {
                stream: "stdin writer",
            })
        }
        None => {
            w.stop_and_join(TEARDOWN_GRACE)?;
            Err(ExecutionError::PipesHeld { after: grace })
        }
    }
}

/// 错误返回前的统一收尾（F-816-R3-02）：先停止自有 reader/writer（有界
/// join，结果不覆盖原始错误），再让调用方传播原错误——任何返回路径都
/// 不带活的自有线程。
fn finish_early(
    err: ExecutionError,
    stdout_w: &mut Option<CaptureWorker>,
    stderr_w: &mut Option<CaptureWorker>,
    writer_w: &mut Option<WriterWorker>,
) -> ExecutionError {
    if let Some(w) = stdout_w.as_mut() {
        w.request_stop();
        w.mark_owned_closed();
        let _ = w.stop_and_join(TEARDOWN_GRACE);
    }
    if let Some(w) = stderr_w.as_mut() {
        w.request_stop();
        w.mark_owned_closed();
        let _ = w.stop_and_join(TEARDOWN_GRACE);
    }
    if let Some(w) = writer_w.as_mut() {
        w.request_stop();
        let _ = w.stop_and_join(TEARDOWN_GRACE);
    }
    err
}

/// 拆分超时路径的捕获结果：I/O 结束具名传播（不被 Timeout 吞掉，
/// F-816-R2-06），其余结束保留字节。
fn take_capture_bytes(
    stream: &'static str,
    cap: Option<Capture>,
) -> Result<Option<Vec<u8>>, ExecutionError> {
    match cap {
        None => Ok(None),
        Some(Capture {
            bytes: _,
            end: ReaderEnd::Io(source),
        }) => Err(ExecutionError::ReaderIo { stream, source }),
        Some(Capture { bytes, end: _ }) => Ok(Some(bytes)),
    }
}

/// 无显式 deadline 时，根退出后等待管道收尾的默认宽限;超时即具名
/// [`ExecutionError::PipesHeld`]——不无限等待(F-816-R1-02)。
const POST_EXIT_COLLECT_GRACE: Duration = Duration::from_secs(10);

/// 停止自有 reader/writer 后等待其真停的宽限(stop 生效延迟 ≤ 轮询切片,
/// 正常远小于本值)。
const TEARDOWN_GRACE: Duration = Duration::from_secs(3);

/// 一次性执行到终态(含可选 deadline)。deadline 覆盖**整次**执行:
/// stdin 写入、根等待与资源收尾共用同一绝对截止(F-816-R1-01)。
/// 根退出后、reader 收尾前,仍持管道的 owned 后代先被终止
/// (F-816-R1-02);**任何返回路径都不带活的自有线程**:超时/关闭先请求
/// 停止、终止 owned 树、join 确认真停;reader 读错/panic、writer 写错与
/// 终止失败在正常/超时/关闭各路径具名传播(F-816-R1-06/R2-06);
/// `elapsed` 取实际返回时刻(F-816-R2-06)。
pub fn run(spec: &ExecutionSpec) -> Result<ExecutionResult, ExecutionError> {
    validate(spec)?;
    let start = Instant::now();
    let deadline_at = spec.deadline.map(|limit| start + limit);
    let mut proc = spawn(spec)?;

    // reader 先于任何写入启动:双向大流量不会因对方先写满而互等。
    let mut stdout_w = proc
        .take_stdout()
        .map(|r| CaptureWorker::spawn("stdout", r));
    let mut stderr_w = proc
        .take_stderr()
        .map(|r| CaptureWorker::spawn("stderr", r));

    let mut writer_w = match spec.stdin_data.clone() {
        Some(data) => {
            let stdin = proc
                .take_stdin()
                .ok_or_else(|| ExecutionError::InvalidSpec("stdin pipe missing".into()))?;
            Some(WriterWorker::spawn(stdin, data))
        }
        None => None,
    };

    let mut timed_out = false;
    let status = match deadline_at {
        Some(at) => {
            let remaining = at.saturating_duration_since(Instant::now());
            match proc.wait_deadline(remaining) {
                Ok(Some(status)) => status,
                Ok(None) => {
                    timed_out = true;
                    // 终止 owned 树;收尸与失败传播在停止自有线程之后统一进行。
                    let _ = proc.terminate();
                    match proc.child.wait() {
                        Ok(status) => status,
                        Err(source) => {
                            let err = ExecutionError::Io {
                                stage: "wait",
                                source,
                            };
                            return Err(finish_early(
                                err,
                                &mut stdout_w,
                                &mut stderr_w,
                                &mut writer_w,
                            ));
                        }
                    }
                }
                Err(source) => {
                    let err = ExecutionError::Io {
                        stage: "wait",
                        source,
                    };
                    return Err(finish_early(
                        err,
                        &mut stdout_w,
                        &mut stderr_w,
                        &mut writer_w,
                    ));
                }
            }
        }
        None => match proc.child.wait() {
            Ok(status) => status,
            Err(source) => {
                let err = ExecutionError::Io {
                    stage: "wait",
                    source,
                };
                return Err(finish_early(
                    err,
                    &mut stdout_w,
                    &mut stderr_w,
                    &mut writer_w,
                ));
            }
        },
    };
    proc.reaped = true;

    if timed_out {
        // 先请求自有线程停止(非拥有方持管道时也不悬挂),再终止 owned 树
        //(管道关闭同时解除任何阻塞),join 确认真停;随后取消失败与
        // reader/writer 的 Io/panic 具名传播,elapsed 取实际时刻。
        if let Some(w) = &stdout_w {
            w.request_stop();
        }
        if let Some(w) = &stderr_w {
            w.request_stop();
        }
        if let Some(w) = &writer_w {
            w.request_stop();
        }
        let cancel_result = proc.cancel();
        // owned 写端已全关(树已终止):reader 的 EOF 判定依据(Windows)。
        if let Some(w) = &stdout_w {
            w.mark_owned_closed();
        }
        if let Some(w) = &stderr_w {
            w.mark_owned_closed();
        }
        // 三个线程都确认真停之后才传播任何失败(先清理后报告)。
        let out_r = stdout_w.as_mut().map(|w| w.stop_and_join(TEARDOWN_GRACE));
        let err_r = stderr_w.as_mut().map(|w| w.stop_and_join(TEARDOWN_GRACE));
        let wr_r = writer_w.as_mut().map(|w| w.stop_and_join(TEARDOWN_GRACE));
        let elapsed = start.elapsed();
        if let Err(source) = cancel_result {
            return Err(ExecutionError::Io {
                stage: "cancel after timeout",
                source,
            });
        }
        let out_cap = out_r.transpose()?;
        let err_cap = err_r.transpose()?;
        let wr_out = wr_r.transpose()?;
        let stdout = take_capture_bytes("stdout", out_cap)?;
        let stderr = take_capture_bytes("stderr", err_cap)?;
        if let Some(WriterOutcome {
            end: WriterEnd::Io(source),
            ..
        }) = wr_out
        {
            // owned 终止使阻塞写以 BrokenPipe 解除属拆除噪声(deadline
            // 已先行分类);其余写失败具名传播,不以 Timeout 掩盖。
            let teardown = matches!(
                source.kind(),
                io::ErrorKind::BrokenPipe | io::ErrorKind::WriteZero | io::ErrorKind::NotConnected
            );
            if !teardown {
                return Err(ExecutionError::Io {
                    stage: "write stdin",
                    source,
                });
            }
        }
        return Ok(ExecutionResult {
            kind: ExitKind::Timeout { after: elapsed },
            stdout,
            stderr,
            elapsed,
        });
    }

    // 根已退出(真实分类保留):先终止仍存活的 owned 后代——它们可能
    // 持有输出管道使 reader 等 EOF(F-816-R1-02);Unix 下组空(ESRCH)
    // 视为无事可做。
    if proc.tree_scope == TreeScope::ProcessTree {
        if let Err(source) = proc.terminate() {
            let err = ExecutionError::Io {
                stage: "terminate after root exit",
                source,
            };
            return Err(finish_early(
                err,
                &mut stdout_w,
                &mut stderr_w,
                &mut writer_w,
            ));
        }
    }
    // 根已退出(+树已终止):owned 写端全部关闭——reader 的 EOF 判定依据。
    if let Some(w) = &stdout_w {
        w.mark_owned_closed();
    }
    if let Some(w) = &stderr_w {
        w.mark_owned_closed();
    }
    let grace = deadline_at
        .map(|at| at.saturating_duration_since(Instant::now()))
        .unwrap_or(POST_EXIT_COLLECT_GRACE);
    // 全部收尾完成后才传播任一失败（各 finish 内部已停自身线程,
    // 顺序化保证错误返回时三线程都不再存活——F-816-R3-02）。
    let stdout = finish_capture(&mut stdout_w, grace);
    let stderr = finish_capture(&mut stderr_w, grace);
    let writer = finish_writer(&mut writer_w, grace);
    let stdout = stdout?;
    let stderr = stderr?;
    writer?;

    Ok(ExecutionResult {
        kind: ExitKind::from_status(&status),
        stdout,
        stderr,
        elapsed: start.elapsed(),
    })
}

/// owned 进程句柄:终止(直接/树)、收尸、管道访问、带限等待。
pub struct OwnedProcess {
    child: Child,
    #[cfg(windows)]
    job: Option<win_job::JobHandle>,
    #[allow(dead_code)] // Unix DirectChildOnly 路径不读;保留以对称记录归属
    tree_scope: TreeScope,
    reaped: bool,
}

impl OwnedProcess {
    pub fn id(&self) -> u32 {
        self.child.id()
    }

    /// 本次执行的 owned 树范围（shutdown/收尾方决定终止策略）。
    pub fn tree_scope(&self) -> TreeScope {
        self.tree_scope
    }

    pub fn stdin(&mut self) -> Option<&mut ChildStdin> {
        self.child.stdin.as_mut()
    }

    pub fn stdout(&mut self) -> Option<&mut ChildStdout> {
        self.child.stdout.as_mut()
    }

    pub fn stderr(&mut self) -> Option<&mut ChildStderr> {
        self.child.stderr.as_mut()
    }

    /// 移交 stdin 管道所有权(取出后 drop 即向子进程发 EOF;shutdown 语义
    /// 由消费方拥有)。T-12 客户端消费:长运行协议宿主的生命期归调用方。
    pub fn take_stdin(&mut self) -> Option<ChildStdin> {
        self.child.stdin.take()
    }

    /// 移交 stdout 管道所有权(消费方自带 reader 线程时;设施 `run` 的
    /// 内置 reader 不会重复存在——spawn 路径不创建线程)。
    pub fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.child.stdout.take()
    }

    /// 移交 stderr 管道所有权(同 [`OwnedProcess::take_stdout`])。
    pub fn take_stderr(&mut self) -> Option<ChildStderr> {
        self.child.stderr.take()
    }

    /// 终止 owned 进程(树):Windows job 树 / Unix 进程组 / 直接 kill。
    /// 只发出终止,不收尸(收尸见 [`OwnedProcess::wait`])。
    pub fn terminate(&mut self) -> io::Result<()> {
        #[cfg(windows)]
        {
            if let Some(job) = &self.job {
                return job.terminate();
            }
        }
        #[cfg(unix)]
        if self.tree_scope == TreeScope::ProcessTree {
            return unix_pg::kill_group(self.child.id());
        }
        self.child.kill()
    }

    /// cancel = terminate + wait 收尸;返回最终状态。
    /// Windows 下树终止码为 1(job)/std kill 码——分类请用调用方上下文
    /// ([`ExitKind::Cancelled`]),不要读 code 判成功失败。
    pub fn cancel(&mut self) -> io::Result<ExitStatus> {
        let terminate_result = self.terminate();
        let status = self.child.wait()?;
        self.reaped = true;
        terminate_result?;
        Ok(status)
    }

    /// 阻塞收尸。
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        let status = self.child.wait()?;
        self.reaped = true;
        Ok(status)
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        let status = self.child.try_wait()?;
        if status.is_some() {
            self.reaped = true;
        }
        Ok(status)
    }

    /// 带限等待:Ok(None) = 限期已到仍在运行(未终止、未收尸,调用方
    /// 决定 cancel 或续等)。轮询粒度 ≤25ms。
    pub fn wait_deadline(&mut self, limit: Duration) -> io::Result<Option<ExitStatus>> {
        let deadline = Instant::now() + limit;
        loop {
            if let Some(status) = self.child.try_wait()? {
                self.reaped = true;
                return Ok(Some(status));
            }
            let now = Instant::now();
            if now >= deadline {
                return Ok(None);
            }
            let poll = (deadline - now)
                .min(Duration::from_millis(25))
                .max(Duration::from_millis(1));
            std::thread::sleep(poll);
        }
    }
}

impl Drop for OwnedProcess {
    fn drop(&mut self) {
        // 未收尸 ⇒ 兜底终止+收尸(镜像 pkg.rs ChildGuard 语义;Windows
        // job 句柄关闭本身即杀全树,这里仍显式收尸观察停止)。
        if !self.reaped {
            let _ = self.terminate();
            let _ = self.child.wait();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- F-816-R1-06 故障注入面:reader 读错/panic/管道被持有 ----

    struct Scripted {
        chunks: Vec<Vec<u8>>,
        tail: Tail,
    }

    enum Tail {
        Eof,
        Err(&'static str),
        /// 永不结束(模拟管道写端被非拥有进程持有)。
        Block,
    }

    impl io::Read for Scripted {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if let Some(chunk) = self.chunks.first() {
                // 测试分块恒小于一次读取窗口,整块交付。
                let n = chunk.len().min(buf.len());
                buf[..n].copy_from_slice(&chunk[..n]);
                self.chunks.remove(0);
                return Ok(n);
            }
            match self.tail {
                Tail::Eof => Ok(0),
                Tail::Err(msg) => Err(io::Error::other(msg)),
                // 模拟管道写端被非拥有进程持有:读永不返回(测试线程
                // 随进程终结;collect 侧以小宽限判 PipesHeld)。
                Tail::Block => loop {
                    std::thread::sleep(Duration::from_secs(3600));
                },
            }
        }
    }

    // 假读端即刻"就绪"(真管道的就绪等待由平台 impl 承担;注入面只
    // 关心 read 语义)。
    impl PipePoll for Scripted {
        fn poll_ready(&self, _slice: Duration) -> bool {
            true
        }
    }

    /// reader 读错 → ReaderIo 具名传播(不是静默 EOF/Success)。
    #[test]
    fn capture_reader_io_error_is_named() {
        let mut w = Some(CaptureWorker::spawn(
            "stdout",
            Scripted {
                chunks: vec![b"partial".to_vec()],
                tail: Tail::Err("kernel read gone"),
            },
        ));
        let err = finish_capture(&mut w, Duration::from_secs(2)).unwrap_err();
        match err {
            ExecutionError::ReaderIo { stream, source } => {
                assert_eq!(stream, "stdout");
                assert!(source.to_string().contains("kernel read gone"));
            }
            other => panic!("expected ReaderIo, got {other:?}"),
        }
    }

    /// reader panic(通道断开、无消息)→ 停止回收后 ReaderPanicked 具名传播。
    #[test]
    fn capture_reader_panic_is_named() {
        struct Panic;
        impl io::Read for Panic {
            fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
                panic!("injected reader panic");
            }
        }
        impl PipePoll for Panic {
            fn poll_ready(&self, _slice: Duration) -> bool {
                true
            }
        }
        let mut w = Some(CaptureWorker::spawn("stdout", Panic));
        let err = finish_capture(&mut w, Duration::from_secs(2)).unwrap_err();
        assert!(
            matches!(err, ExecutionError::ReaderPanicked { stream: "stdout" }),
            "{err:?}"
        );
    }

    /// 管道写端被非拥有进程持有(reader 无 EOF)→ 停止自有线程后具名
    /// PipesHeld,不无限等待、不谎报成功、不带活线程返回。
    #[test]
    fn pipes_held_is_named_after_grace() {
        let mut w = Some(CaptureWorker::spawn(
            "stdout",
            Scripted {
                chunks: vec![],
                tail: Tail::Block,
            },
        ));
        let err = finish_capture(&mut w, Duration::from_millis(150)).unwrap_err();
        assert!(matches!(err, ExecutionError::PipesHeld { .. }), "{err:?}");
    }

    /// F-816-R3-01：stop 语义=先排空已可读内容——预置数据+先行置 stop，
    /// 字节不得丢弃（确定性，无 sleep/无负载依赖）。
    #[test]
    fn stop_drains_readable_bytes_before_stopping() {
        let payload = b"arrived-before-stop".to_vec();
        let mut w = Some(CaptureWorker::spawn(
            "stdout",
            Scripted {
                chunks: vec![payload.clone()],
                tail: Tail::Eof,
            },
        ));
        let cap = w
            .as_mut()
            .unwrap()
            .stop_and_join(Duration::from_secs(2))
            .expect("worker stops");
        assert!(matches!(cap.end, ReaderEnd::Stopped), "{:?}", cap.end);
        assert_eq!(
            cap.bytes, payload,
            "arrived bytes must not be dropped on stop"
        );
    }

    /// 干净 EOF 对照:字节完整保留。
    #[test]
    fn capture_clean_eof_keeps_bytes() {
        let mut w = Some(CaptureWorker::spawn(
            "stdout",
            Scripted {
                chunks: vec![b"hello".to_vec(), b" world".to_vec()],
                tail: Tail::Eof,
            },
        ));
        let out = finish_capture(&mut w, Duration::from_secs(2))
            .unwrap()
            .unwrap();
        assert_eq!(out, b"hello world");
    }

    /// F-816-R2-06 核心:**真实管道**、写端被持有(不关)——stop_and_join
    /// 在宽限内使自有 reader 真停(Stopped,字节截至停止点),不悬挂。
    #[test]
    fn stop_and_join_stops_reader_on_held_real_pipe() {
        let (reader, keeper) = std::io::pipe().expect("pipe");
        let mut w = CaptureWorker::spawn("stdout", reader);
        assert!(
            w.wait(Duration::from_millis(150)).is_none(),
            "no EOF while the write end is held"
        );
        let cap = w
            .stop_and_join(Duration::from_secs(2))
            .expect("reader must stop after the stop flag");
        assert!(matches!(cap.end, ReaderEnd::Stopped), "{:?}", cap.end);
        drop(keeper); // 释放持有的写端,管道资源随作用域回收。
    }

    /// writer 对称面:真实管道、读端被持有、写入量超缓冲——Windows 同步
    /// 写端阻塞中的分片写不能被 stop 中断(内核限制;生产路径由 owned
    /// 终止解除)。合同在此场景的承诺是有界+具名:stop_and_join 不得
    /// 悬挂,宽限内未停即 PipesHeld。Unix(poll POLLOUT)下 stop 真停,
    /// 返回 Stopped——两种结果都接受,断言共同不变量(不悬挂、不谎报)。
    #[test]
    fn writer_on_held_pipe_is_bounded_and_named() {
        let (keeper, writer_end) = std::io::pipe().expect("pipe");
        let data = vec![b'w'; 1024 * 1024];
        let mut w = WriterWorker::spawn(writer_end, data.clone());
        assert!(
            w.wait(Duration::from_millis(200)).is_none(),
            "writer cannot finish while the read end is held"
        );
        let started = Instant::now();
        let outcome = w.stop_and_join(Duration::from_secs(2));
        let elapsed = started.elapsed();
        assert!(
            elapsed < Duration::from_secs(4),
            "stop_and_join must be bounded (elapsed={elapsed:?})"
        );
        match outcome {
            Ok(out) => {
                assert!(matches!(out.end, WriterEnd::Stopped), "{:?}", out.end);
                assert!(out.written < data.len(), "partial write before stop");
            }
            Err(ExecutionError::PipesHeld { .. }) => {}
            other => panic!("expected Stopped or named PipesHeld, got {other:?}"),
        }
        drop(keeper);
    }

    #[test]
    fn empty_argv_rejected() {
        let err = run(&ExecutionSpec::default()).unwrap_err();
        assert!(matches!(err, ExecutionError::InvalidSpec(_)), "{err}");
    }

    #[test]
    fn stdin_data_requires_piped_stdin() {
        let mut spec = ExecutionSpec::new(["definitely-missing-te"]);
        spec.stdin_data = Some(vec![1]);
        spec.stdin = StdioPolicy::Null;
        let err = run(&spec).unwrap_err();
        assert!(matches!(err, ExecutionError::InvalidSpec(_)), "{err}");
    }

    #[cfg(not(windows))]
    #[test]
    fn cmd_shim_rejected_off_windows() {
        let mut spec = ExecutionSpec::new(["sh"]);
        spec.windows_cmd_shim = true;
        let err = run(&spec).unwrap_err();
        assert!(matches!(err, ExecutionError::InvalidSpec(_)), "{err}");
    }
}

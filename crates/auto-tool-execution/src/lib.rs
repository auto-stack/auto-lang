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
use std::path::PathBuf;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
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

/// reader 结束方式(具名;干净 EOF 与 I/O 错误分开——F-816-R1-06)。
#[derive(Debug)]
enum ReaderEnd {
    Eof,
    Io(io::Error),
}

/// 捕获流的产出:字节 + 结束方式。线程 panic 时通道断开、无消息。
struct Capture {
    bytes: Vec<u8>,
    end: ReaderEnd,
}

/// 捕获 reader 的完成通道(有界等待;`Disconnected` = 线程 panic)。
type CaptureRx = std::sync::mpsc::Receiver<Capture>;

fn capture_thread<R: io::Read + Send + 'static>(mut r: R) -> CaptureRx {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let mut end = ReaderEnd::Eof;
        let mut chunk = [0u8; 16 * 1024];
        loop {
            match r.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => bytes.extend_from_slice(&chunk[..n]),
                Err(e) => {
                    end = ReaderEnd::Io(e);
                    break;
                }
            }
        }
        let _ = tx.send(Capture { bytes, end });
    });
    rx
}

/// 正常路径的捕获收集:EOF 才算成功;I/O 错误/panic/宽限内无 EOF(管道
/// 被**非本次拥有**的进程持有)分别具名失败,不以部分字节谎报成功。
fn collect_capture(
    stream: &'static str,
    rx: Option<CaptureRx>,
    grace: Duration,
) -> Result<Option<Vec<u8>>, ExecutionError> {
    match rx {
        None => Ok(None),
        Some(rx) => match rx.recv_timeout(grace) {
            Ok(cap) => match cap.end {
                ReaderEnd::Eof => Ok(Some(cap.bytes)),
                ReaderEnd::Io(source) => Err(ExecutionError::ReaderIo { stream, source }),
            },
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                Err(ExecutionError::PipesHeld { after: grace })
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                Err(ExecutionError::ReaderPanicked { stream })
            }
        },
    }
}

/// owned 树终止后的捕获收集:尽力保留字节,结束状态属拆除噪声(EOF 或
/// torn-down I/O 皆可能),不改变调用方的超时/取消分类。
fn drain_capture_after_stop(rx: Option<CaptureRx>, grace: Duration) -> Option<Vec<u8>> {
    let rx = rx?;
    match rx.recv_timeout(grace) {
        Ok(cap) => Some(cap.bytes),
        Err(_) => None,
    }
}

type WriterRx = std::sync::mpsc::Receiver<io::Result<()>>;

/// stdin 写入线程:对端不读导致管道写阻塞时,不越过调用方 deadline
/// (F-816-R1-01)——超时路径先终止 owned 树,管道关闭使本线程解除阻塞。
fn stdin_writer_thread(mut stdin: ChildStdin, data: Vec<u8>) -> WriterRx {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let result = io::Write::write_all(&mut stdin, &data);
        // drop(stdin) 关闭管道,子进程 stdin 见 EOF。
        let _ = tx.send(result);
    });
    rx
}

fn collect_writer(rx: Option<WriterRx>, grace: Duration) -> Result<(), ExecutionError> {
    match rx {
        None => Ok(()),
        Some(rx) => match rx.recv_timeout(grace) {
            Ok(Ok(())) => Ok(()),
            Ok(Err(source)) => Err(ExecutionError::Io {
                stage: "write stdin",
                source,
            }),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                Err(ExecutionError::PipesHeld { after: grace })
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                Err(ExecutionError::ReaderPanicked {
                    stream: "stdin writer",
                })
            }
        },
    }
}

/// 无显式 deadline 时,根退出后等待管道收尾的默认宽限;超时即具名
/// [`ExecutionError::PipesHeld`]——不无限等待(F-816-R1-02)。
const POST_EXIT_COLLECT_GRACE: Duration = Duration::from_secs(10);

/// owned 树终止后的收尾宽限:拆除后管道应立即关闭,给足调度余量即可。
const TEARDOWN_GRACE: Duration = Duration::from_secs(3);

/// 一次性执行到终态(含可选 deadline)。deadline 覆盖**整次**执行:
/// stdin 写入、根等待与资源收尾共用同一绝对截止(F-816-R1-01)。
/// 根退出后、reader 收尾前,仍持管道的 owned 后代先被终止
/// (F-816-R1-02);reader 的读错/panic 与 writer/终止失败具名传播
/// (F-816-R1-06)。
pub fn run(spec: &ExecutionSpec) -> Result<ExecutionResult, ExecutionError> {
    validate(spec)?;
    let start = Instant::now();
    let deadline_at = spec.deadline.map(|limit| start + limit);
    let mut proc = spawn(spec)?;

    // reader 先于任何写入启动:双向大流量不会因对方先写满而互等。
    let stdout_rx = proc.take_stdout().map(capture_thread);
    let stderr_rx = proc.take_stderr().map(capture_thread);

    let writer_rx = match spec.stdin_data.clone() {
        Some(data) => {
            let stdin = proc
                .take_stdin()
                .ok_or_else(|| ExecutionError::InvalidSpec("stdin pipe missing".into()))?;
            Some(stdin_writer_thread(stdin, data))
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
                    // cancel = terminate owned 树 + 收尸;失败具名传播
                    // (此时树可能仍活,只能放弃 reader/writer 等待)。
                    proc.cancel().map_err(|source| ExecutionError::Io {
                        stage: "cancel after timeout",
                        source,
                    })?
                }
                Err(source) => {
                    return Err(ExecutionError::Io {
                        stage: "wait",
                        source,
                    })
                }
            }
        }
        None => proc.child.wait().map_err(|source| ExecutionError::Io {
            stage: "wait",
            source,
        })?,
    };
    proc.reaped = true;
    let elapsed = start.elapsed();

    if timed_out {
        // owned 树已终止并收尸:管道关闭使 writer/reader 解除阻塞并收尾;
        // 其结束状态属拆除噪声,尽力保留已捕获字节,不改 Timeout 分类。
        let stdout = drain_capture_after_stop(stdout_rx, TEARDOWN_GRACE);
        let stderr = drain_capture_after_stop(stderr_rx, TEARDOWN_GRACE);
        if let Some(rx) = writer_rx {
            // 挂起在满管道上的写线程随树终止解除;丢弃拆除期结果。
            let _ = rx.recv_timeout(TEARDOWN_GRACE);
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
        proc.terminate().map_err(|source| ExecutionError::Io {
            stage: "terminate after root exit",
            source,
        })?;
    }
    let grace = deadline_at
        .map(|at| at.saturating_duration_since(Instant::now()))
        .unwrap_or(POST_EXIT_COLLECT_GRACE);
    let stdout = collect_capture("stdout", stdout_rx, grace)?;
    let stderr = collect_capture("stderr", stderr_rx, grace)?;
    collect_writer(writer_rx, grace)?;

    Ok(ExecutionResult {
        kind: ExitKind::from_status(&status),
        stdout,
        stderr,
        elapsed,
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

    /// reader 读错 → ReaderIo 具名传播(不是静默 EOF/Success)。
    #[test]
    fn capture_reader_io_error_is_named() {
        let rx = capture_thread(Scripted {
            chunks: vec![b"partial".to_vec()],
            tail: Tail::Err("kernel read gone"),
        });
        let err = collect_capture("stdout", Some(rx), Duration::from_secs(2)).unwrap_err();
        match err {
            ExecutionError::ReaderIo { stream, source } => {
                assert_eq!(stream, "stdout");
                assert!(source.to_string().contains("kernel read gone"));
            }
            other => panic!("expected ReaderIo, got {other:?}"),
        }
    }

    /// reader panic(通道断开、无消息)→ ReaderPanicked 具名传播。
    #[test]
    fn capture_reader_panic_is_named() {
        struct Panic;
        impl io::Read for Panic {
            fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
                panic!("injected reader panic");
            }
        }
        let rx = capture_thread(Panic);
        let err = collect_capture("stdout", Some(rx), Duration::from_secs(2)).unwrap_err();
        assert!(
            matches!(err, ExecutionError::ReaderPanicked { stream: "stdout" }),
            "{err:?}"
        );
    }

    /// 管道写端被非拥有进程持有(reader 无 EOF)→ 宽限内具名 PipesHeld,
    /// 不无限等待、不谎报成功。
    #[test]
    fn pipes_held_is_named_after_grace() {
        let rx = capture_thread(Scripted {
            chunks: vec![],
            tail: Tail::Block,
        });
        let err = collect_capture("stdout", Some(rx), Duration::from_millis(150)).unwrap_err();
        assert!(matches!(err, ExecutionError::PipesHeld { .. }), "{err:?}");
    }

    /// 干净 EOF 对照:字节完整保留。
    #[test]
    fn capture_clean_eof_keeps_bytes() {
        let rx = capture_thread(Scripted {
            chunks: vec![b"hello".to_vec(), b" world".to_vec()],
            tail: Tail::Eof,
        });
        let out = collect_capture("stdout", Some(rx), Duration::from_secs(2))
            .unwrap()
            .unwrap();
        assert_eq!(out, b"hello world");
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

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
//! - [`StdioPolicy::Inherit`] + [`TreeScope::ProcessGroup`] 在 Unix 下使
//!   子进程脱离终端前台组(Ctrl+C 不再直达),交互式命令保持
//!   [`TreeScope::DirectChildOnly`]。

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
    /// argv[0] = 程序,其余为参数;零 shell 解释(空 argv 报
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
    /// 便捷构造:argv[0] 为程序,其余为参数。
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
        }
    }
}

impl std::error::Error for ExecutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ExecutionError::Spawn { source, .. } | ExecutionError::Io { source, .. } => {
                Some(source)
            }
            ExecutionError::InvalidSpec(_) => None,
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
            // 立即入 job:孙进程只有被子进程 spawn 后才会存在,而本调用
            // 先于子进程用户代码执行(测试侧以"观察到孙进程存活再取消"
            // 的协议确定性关闭 assign 竞态窗口)。
            match win_job::create_and_assign(&proc.child) {
                Ok(job) => proc.job = Some(job),
                Err(source) => {
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

/// 一次性执行到终态(含可选 deadline)。捕获策略为 Piped 的流由内部
/// reader 线程并发读取(大输出不死锁),并在进程终止、管道 EOF 后 join
/// ——reader 真终止后才返回。
pub fn run(spec: &ExecutionSpec) -> Result<ExecutionResult, ExecutionError> {
    validate(spec)?;
    let start = Instant::now();
    let mut proc = spawn(spec)?;

    if let Some(data) = spec.stdin_data.clone() {
        let mut stdin = proc
            .child
            .stdin
            .take()
            .ok_or_else(|| ExecutionError::InvalidSpec("stdin pipe missing".into()))?;
        io::Write::write_all(&mut stdin, &data).map_err(|source| ExecutionError::Io {
            stage: "write stdin",
            source,
        })?;
        // drop 关闭管道,子进程 stdin 见 EOF。
    }

    let stdout_reader = proc
        .child
        .stdout
        .take()
        .map(reader_thread::<std::process::ChildStdout>);
    let stderr_reader = proc
        .child
        .stderr
        .take()
        .map(reader_thread::<std::process::ChildStderr>);

    let mut decided: Option<ExitKind> = None;
    let status = match spec.deadline {
        Some(limit) => match proc
            .wait_deadline(limit)
            .map_err(|source| ExecutionError::Io {
                stage: "wait",
                source,
            })? {
            Some(status) => status,
            None => {
                // cancel 内含 terminate + wait 收尸;超时分类由上下文给出。
                let status = proc.cancel().map_err(|source| ExecutionError::Io {
                    stage: "cancel after timeout",
                    source,
                })?;
                decided = Some(ExitKind::Timeout {
                    after: start.elapsed(),
                });
                status
            }
        },
        None => proc.child.wait().map_err(|source| ExecutionError::Io {
            stage: "wait",
            source,
        })?,
    };
    proc.reaped = true;

    let stdout = stdout_reader.map(join_reader);
    let stderr = stderr_reader.map(join_reader);

    Ok(ExecutionResult {
        kind: decided.unwrap_or_else(|| ExitKind::from_status(&status)),
        stdout,
        stderr,
        elapsed: start.elapsed(),
    })
}

fn reader_thread<R: io::Read + Send + 'static>(mut r: R) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 16 * 1024];
        loop {
            match r.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => buf.extend_from_slice(&chunk[..n]),
                Err(_) => break,
            }
        }
        buf
    })
}

fn join_reader(h: std::thread::JoinHandle<Vec<u8>>) -> Vec<u8> {
    h.join().unwrap_or_default()
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

    pub fn stdin(&mut self) -> Option<&mut ChildStdin> {
        self.child.stdin.as_mut()
    }

    pub fn stdout(&mut self) -> Option<&mut ChildStdout> {
        self.child.stdout.as_mut()
    }

    pub fn stderr(&mut self) -> Option<&mut ChildStderr> {
        self.child.stderr.as_mut()
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

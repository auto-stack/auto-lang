//! PLAN-816 T-07：真实 CLI 进程 driver（V-01/V-04 的可自动化切片）。
//!
//! 以 `CARGO_BIN_EXE_auto` 驱动**真实二进制**（真 CLI→语言/管理/服务/
//! 工具链路，非单元层模拟）：
//! - V-01 裸脚本/argv 透传（`--` 后强制透传，Plan 524 契约）；
//! - V-04 serve/req 具名管道往返（daemon 协议真实入口）；
//! - V-04 watch 变更触发 SFC 再生（HMR 前置）+ 终止退出；
//! - V-04 gen 经真实工具进程（I-TOOL 五操作链，AUTO_UI_TOOL_BIN 指向
//!   主 workspace 产物；缺工具=诚实红灯）。
//!
//! F-816-B-R5-04：全部自有 reader（run_bounded 排水与 WatchGuard 泵）
//! 走 [`stoppable_drain`] 的**双平台真停止协议**——Windows=停止侧重发
//! CancelIoEx 唤醒挂起读；Unix=poll(2) 50ms 就绪分片（阻塞 read 在无
//! 字节/无 EOF 时永远观察不到停止标志）。宽限尽未停的 JoinHandle 移入
//! 进程级残留登记（归属保留），不随局部 Vec 销毁、不伪称已回收。

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const AUTO: &str = env!("CARGO_BIN_EXE_auto");

fn run(args: &[&str], cwd: &Path, env: &[(&str, String)]) -> (i32, String, String) {
    let mut cmd = Command::new(AUTO);
    cmd.args(args).current_dir(cwd).stdin(Stdio::null());
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd
        .output()
        .unwrap_or_else(|e| panic!("spawn auto {:?}: {e}", args));
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// 可停止排水/行泵（F-816-B-R4/R5-04）。双平台真停止：
/// - Windows：读线程阻塞在 ReadFile；停止侧置标志后**切片重发
///   CancelIoEx** 直至真停——外部进程经 DuplicateHandle 持写端时，杀
///   owned 树关不掉该端，取消我方挂起读是唯一有界手段。
/// - Unix：poll(2) 50ms 就绪分片——分片间恒可观察停止（无数据时 poll
///   超时而非阻塞；SCM_RIGHTS 外部持有者也不能把线程钉在内核读上）。
/// 宽限尽未停：JoinHandle 移入进程级 RESIDUE 登记（归属保留），随管道
/// 最终关闭线程自然退出；不丢弃、不 detach。
mod stoppable_drain {
    use std::io::Read;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    /// 宽限尽仍活线程的归属登记（进程级；测试进程结束前不丢弃）。
    static RESIDUE: Mutex<Vec<std::thread::JoinHandle<()>>> = Mutex::new(Vec::new());

    /// RESIDUE 登记数（F-816-B-R7-04 永久回归 oracle）：返回前自有
    /// reader 应真停收割（join 清归属），登记数 0；>0 = 有线程只在全局
    /// 表里"保留归属"而未在返回前真停。
    pub fn residue_len() -> usize {
        RESIDUE.lock().unwrap().len()
    }

    #[allow(dead_code)]
    pub struct Drain {
        handle: Option<std::thread::JoinHandle<()>>,
        stop: Arc<AtomicBool>,
        /// 已收字节（字节泵）；行泵不用（行直接走通道）。
        pub buf: Arc<Mutex<String>>,
        #[cfg(windows)]
        cancel_target: isize,
        /// Unix 轮询 fd（诊断身份；泵线程闭包另持同一 fd 数字与管道对象）。
        #[cfg(unix)]
        fd: i32,
    }

    impl Drain {
        pub fn alive(&self) -> bool {
            self.handle
                .as_ref()
                .map(|h| !h.is_finished())
                .unwrap_or(false)
        }

        /// 有界停止：置标志 →（Win）切片重发取消 → 确认真停后 join 收割；
        /// 宽限尽=JoinHandle 移入 RESIDUE（false），归属不丢。
        pub fn stop_bounded(&mut self, grace: Duration) -> bool {
            self.stop.store(true, Ordering::Relaxed);
            if let Some(join) = self.handle.as_ref() {
                let deadline = Instant::now() + grace;
                while !join.is_finished() && Instant::now() < deadline {
                    #[cfg(windows)]
                    cancel_pending(self);
                    #[cfg(unix)]
                    {
                        // poll 分片在 ≤50ms 内观察停止——空转等待即可。
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
            match self.handle.take() {
                Some(join) if join.is_finished() => join.join().is_ok(),
                Some(join) => {
                    RESIDUE.lock().unwrap().push(join);
                    false
                }
                None => true,
            }
        }
    }

    #[cfg(windows)]
    fn cancel_pending(d: &Drain) {
        extern "system" {
            fn CancelIoEx(h: isize, overlapped: *const u8) -> i32;
        }
        unsafe {
            CancelIoEx(d.cancel_target, std::ptr::null());
        }
    }

    /// 字节排水（run_bounded 的 stdout/stderr）。
    #[cfg(windows)]
    pub fn spawn_drain<R>(r: R) -> Drain
    where
        R: Read + std::os::windows::io::AsRawHandle + Send + 'static,
    {
        let raw = r.as_raw_handle() as isize;
        build_drain(r, raw, 0)
    }

    /// 字节排水（Unix）。
    #[cfg(unix)]
    pub fn spawn_drain<R>(r: R) -> Drain
    where
        R: Read + std::os::unix::io::AsRawFd + Send + 'static,
    {
        let fd = r.as_raw_fd();
        build_drain(r, 0, fd)
    }

    /// 统一构造：平台原始标识（win_raw/unix fd）在调用点捕获。
    #[allow(clippy::too_many_arguments)]
    fn build_drain<R>(r: R, win_raw: isize, unix_fd: i32) -> Drain
    where
        R: Read + Send + 'static,
    {
        let buf: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let sink = buf.clone();
        #[cfg(windows)]
        {
            let _ = unix_fd;
            let handle = std::thread::Builder::new()
                .spawn(move || pump_win(r, sink, flag, None))
                .expect("spawn drain");
            Drain {
                handle: Some(handle),
                stop,
                buf,
                cancel_target: win_raw,
            }
        }
        #[cfg(unix)]
        {
            let _ = win_raw;
            let handle = std::thread::Builder::new()
                .spawn(move || {
                    // F-816-B-R6-04：持有管道读端对象——fd 数字不保活，
                    // 对象被 drop 即关闭读端（线程随后 read=EBADF/exit120）。
                    let _hold_pipe = r;
                    pump_unix(unix_fd, sink, flag, None)
                })
                .expect("spawn drain");
            Drain {
                handle: Some(handle),
                stop,
                buf,
                fd: unix_fd,
            }
        }
    }

    /// 行泵（WatchGuard：完整行经通道交付；不完整尾部留在内部缓冲）。
    #[cfg(windows)]
    pub fn spawn_lines<R>(r: R, tx: std::sync::mpsc::Sender<String>) -> Drain
    where
        R: Read + std::os::windows::io::AsRawHandle + Send + 'static,
    {
        let raw = r.as_raw_handle() as isize;
        build_lines(r, raw, 0, tx)
    }

    /// 行泵（Unix）。
    #[cfg(unix)]
    pub fn spawn_lines<R>(r: R, tx: std::sync::mpsc::Sender<String>) -> Drain
    where
        R: Read + std::os::unix::io::AsRawFd + Send + 'static,
    {
        let fd = r.as_raw_fd();
        build_lines(r, 0, fd, tx)
    }

    fn build_lines<R>(
        r: R,
        win_raw: isize,
        unix_fd: i32,
        tx: std::sync::mpsc::Sender<String>,
    ) -> Drain
    where
        R: Read + Send + 'static,
    {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let buf = Arc::new(Mutex::new(String::new()));
        #[cfg(windows)]
        {
            let _ = unix_fd;
            let handle = std::thread::Builder::new()
                .spawn(move || pump_win(r, Arc::new(Mutex::new(String::new())), flag, Some(tx)))
                .expect("spawn line pump");
            Drain {
                handle: Some(handle),
                stop,
                buf,
                cancel_target: win_raw,
            }
        }
        #[cfg(unix)]
        {
            let _ = win_raw;
            let handle = std::thread::Builder::new()
                .spawn(move || {
                    // F-816-B-R6-04：同上——行泵也须持有管道对象。
                    let _hold_pipe = r;
                    pump_unix(unix_fd, Arc::new(Mutex::new(String::new())), flag, Some(tx))
                })
                .expect("spawn line pump");
            Drain {
                handle: Some(handle),
                stop,
                buf,
                fd: unix_fd,
            }
        }
    }

    /// Windows：阻塞读（数据就绪即返）；停止侧 CancelIoEx 重发唤醒
    /// （B-R5 已验外部持有者场景 ~207ms 真停）。行模式把完整行送通道。
    #[cfg(windows)]
    fn pump_win<R: Read>(
        mut r: R,
        buf: Arc<Mutex<String>>,
        stop: Arc<AtomicBool>,
        lines: Option<std::sync::mpsc::Sender<String>>,
    ) {
        let mut pending: Vec<u8> = Vec::new();
        loop {
            if stop.load(Ordering::Relaxed) {
                break;
            }
            let mut chunk = [0u8; 4096];
            match r.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => match &lines {
                    Some(tx) => {
                        pending.extend_from_slice(&chunk[..n]);
                        while let Some(pos) = pending.iter().position(|&b| b == b'\n') {
                            let line: Vec<u8> = pending.drain(..=pos).collect();
                            let _ = tx.send(String::from_utf8_lossy(&line).into_owned());
                        }
                    }
                    None => buf
                        .lock()
                        .unwrap()
                        .push_str(&String::from_utf8_lossy(&chunk[..n])),
                },
            }
        }
    }

    /// Unix：poll(2) 50ms 就绪分片 + 就绪后复查停止再读。外部持有者
    /// （SCM_RIGHTS）不能把线程钉在内核读上——分片间恒可观察停止。
    #[cfg(unix)]
    fn pump_unix(
        fd: i32,
        buf: Arc<Mutex<String>>,
        stop: Arc<AtomicBool>,
        lines: Option<std::sync::mpsc::Sender<String>>,
    ) {
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
        let mut pollfd = PollFd {
            fd,
            events: POLLIN,
            revents: 0,
        };
        let mut pending: Vec<u8> = Vec::new();
        loop {
            if stop.load(Ordering::Relaxed) {
                break;
            }
            let ready = unsafe { poll(&mut pollfd, 1, 50) };
            if ready <= 0 {
                continue; // 超时/信号——停止在下一片观察
            }
            if stop.load(Ordering::Relaxed) {
                break;
            }
            let mut chunk = [0u8; 4096];
            let n = unsafe { read(fd, chunk.as_mut_ptr(), chunk.len()) };
            if n <= 0 {
                break; // EOF / 错误
            }
            match &lines {
                Some(tx) => {
                    pending.extend_from_slice(&chunk[..n as usize]);
                    while let Some(pos) = pending.iter().position(|&b| b == b'\n') {
                        let line: Vec<u8> = pending.drain(..=pos).collect();
                        let _ = tx.send(String::from_utf8_lossy(&line).into_owned());
                    }
                }
                None => buf
                    .lock()
                    .unwrap()
                    .push_str(&String::from_utf8_lossy(&chunk[..n as usize])),
            }
        }
    }
}

struct TempProj(PathBuf);

impl TempProj {
    fn new(tag: &str, pac: &str, app_at: &str) -> TempProj {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("p816-driver-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(dir.join("src/front")).unwrap();
        std::fs::write(dir.join("pac.at"), pac).unwrap();
        std::fs::write(dir.join("src/front/app.at"), app_at).unwrap();
        TempProj(dir)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempProj {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const APP: &str = "widget App {\n    view {\n        col {\n            text \"driver\" {}\n        }\n    }\n}\n";

/// watch 子进程 RAII 守卫（F-816-B-R1-04/R5 残项 A）：设施 ProcessTree
/// 承载 + **行泵走 stoppable_drain 停止协议**——release/Drop 先
/// cancel_within 终止 owned 树，再 stop_bounded（Win 重发取消/Unix 分片
/// 观察）真停泵线程；外部持有写端时不再 5 秒放弃残活 reader。
struct WatchGuard {
    proc: Option<auto_tool_execution::OwnedProcess>,
    drain: Option<stoppable_drain::Drain>,
}

impl WatchGuard {
    fn release(mut self) {
        self.stop_bounded();
    }
    fn stop_bounded(&mut self) {
        if let Some(mut proc) = self.proc.take() {
            let _ = proc.cancel_within(Duration::from_secs(5));
        }
        if let Some(mut d) = self.drain.take() {
            let _ = d.stop_bounded(Duration::from_secs(5));
        }
    }
}

impl Drop for WatchGuard {
    fn drop(&mut self) {
        self.stop_bounded();
    }
}

/// V-01：裸脚本执行 + `--` 后 argv 强制透传。
#[test]
fn bare_script_runs_and_argv_passthrough_survives() {
    let dir = std::env::temp_dir().join(format!("p816-argv-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let script = dir.join("argv.at");
    std::fs::write(&script, "use auto.process\nreturn process.args().len()\n").unwrap();
    let (code, out, err) = run(
        &[script.to_str().unwrap(), "--", "one", "two", "three"],
        &dir,
        &[],
    );
    assert_eq!(code, 0, "stdout={out} stderr={err}");
    let scalar = out
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .and_then(|l| l.trim().trim_matches('"').parse::<usize>().ok())
        .unwrap_or_else(|| panic!("argv length scalar expected, got {out:?}"));
    assert_eq!(scalar, 4, "script path + three passthrough args");
    let _ = std::fs::remove_dir_all(&dir);
}

/// V-04：serve（前台 daemon）+ req 具名管道往返。
#[test]
fn serve_daemon_and_req_roundtrip() {
    let pipe = format!("autovm-p816-driver-{}", std::process::id());
    let mut daemon = Command::new(AUTO)
        .args(["serve", "--foreground", "--pipe-name", &pipe])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn daemon");
    std::thread::sleep(Duration::from_millis(1500));
    let workdir = std::env::temp_dir();
    let (code, out, err) = run_bounded(
        &["req", "--pipe-name", &pipe, "--new-session"],
        &workdir,
        &[],
        Duration::from_secs(20),
    );
    assert_eq!(code, 0, "out={out} err={err}");
    assert!(out.trim().starts_with("ses_"), "session id: {out}");
    let (code, list, err) = run_bounded(
        &["req", "--pipe-name", &pipe, "--list"],
        &workdir,
        &[],
        Duration::from_secs(20),
    );
    assert_eq!(code, 0, "out={list} err={err}");
    assert!(
        list.contains("session_count") && !list.contains("\"session_count\":0"),
        "created session must be listed: {list}"
    );
    let _ = daemon.kill();
    let _ = daemon.wait();
}

/// 收尾保留预算（F-816-B-R7-04）：四个返回出口（超时/wait 失败/根退后
/// 持有者/正常 EOF）的终止树+真停 reader 共用——镜像生产设施
/// `TEARDOWN_GRACE` 语义：deadline 只覆盖等待根退出，收尾各操作
/// （cancel_within 与每个 reader 的 stop_bounded）**分别**以本值为宽限
/// 上界（顺序执行的最坏总收尾 ≈ (1+readers)×本值，非共享总截止；Unix
/// poll 分片 ≤50ms + join 需要真实时间，从已耗尽的 deadline 挤 1ms 会使
/// 返回后 reader 残活 ≤100ms）。断言用宽有限界（1.5s）而非严格和上界。
const TEARDOWN: Duration = Duration::from_millis(300);

/// 有界 run（F-816-B-R1-04/R4/R5-04/R7-04）：设施 ProcessTree 承载——超时/
/// 根退出后 cancel_within 终止整棵 owned 树，随后 stoppable_drain 停止
/// 协议真停自有排水（Win=重发取消；Unix=poll 分片）；持有者在场=
/// timeout 回收分类（不沿根退出码报成功）；输出经共享缓冲保留；预算
/// 协调=deadline 管等待 + [`TEARDOWN`] 管终止/真停/收割——返回前自有
/// reader 已 join 确认真停，宽限尽未停才进 RESIDUE（归属不丢）。
#[allow(clippy::type_complexity)]
fn run_bounded_prog(
    program: &std::ffi::OsStr,
    args: &[&str],
    cwd: &Path,
    env: &[(&str, String)],
    limit: Duration,
) -> (i32, String, String) {
    let mut argv: Vec<std::ffi::OsString> = vec![program.to_os_string()];
    argv.extend(args.iter().map(|a| (*a).into()));
    let mut spec = auto_tool_execution::ExecutionSpec::new(argv);
    spec.cwd = Some(cwd.to_path_buf());
    spec.env = env
        .iter()
        .map(|(k, v)| ((*k).into(), v.clone().into()))
        .collect();
    spec.stdin = auto_tool_execution::StdioPolicy::Null;
    spec.stdout = auto_tool_execution::StdioPolicy::Piped;
    spec.stderr = auto_tool_execution::StdioPolicy::Piped;
    spec.tree_scope = auto_tool_execution::TreeScope::ProcessTree;
    let mut proc =
        auto_tool_execution::spawn(&spec).unwrap_or_else(|e| panic!("spawn auto {:?}: {e}", args));
    let mut drains: Vec<stoppable_drain::Drain> = Vec::new();
    if let Some(r) = proc.take_stdout() {
        drains.push(stoppable_drain::spawn_drain(r));
    }
    if let Some(r) = proc.take_stderr() {
        drains.push(stoppable_drain::spawn_drain(r));
    }
    let deadline = Instant::now() + limit;
    // 等待根退出（绝对到期：预算尽→终止树→真停排水→timeout 分类）。
    let code = loop {
        if Instant::now() >= deadline {
            let cancel_err = proc.cancel_within(TEARDOWN).err();
            let mut stopped = true;
            for d in drains.iter_mut() {
                if !d.stop_bounded(TEARDOWN) {
                    stopped = false;
                }
            }
            let partial_out = drains
                .first()
                .map(|d| d.buf.lock().unwrap().clone())
                .unwrap_or_default();
            let mut msg = format!("timeout after {limit:?}");
            if let Some(e) = cancel_err {
                msg.push_str(&format!("; cancel failed: {e}"));
            }
            if !stopped {
                msg.push_str("; drain threads still alive (RESIDUE registry)");
            }
            return (-1, partial_out, msg);
        }
        match proc.wait_deadline(Duration::from_millis(20)) {
            Ok(Some(status)) => break status.code().unwrap_or(-1),
            Ok(None) => {}
            Err(e) => {
                let _ = proc.cancel_within(TEARDOWN);
                for d in drains.iter_mut() {
                    d.stop_bounded(TEARDOWN);
                }
                let partial_out = drains
                    .first()
                    .map(|d| d.buf.lock().unwrap().clone())
                    .unwrap_or_default();
                return (-1, partial_out, format!("wait failed: {e}"));
            }
        }
    };
    // 根已退出≠树已空：短宽限内未 EOF=根外持有者在场——终止 owned 树 +
    // 停止协议真停排水，分类 timeout 回收；正常短命令排水即刻 EOF。
    let grace_end = Instant::now() + Duration::from_millis(150);
    while Instant::now() < grace_end && drains.iter().any(|d| d.alive()) {
        std::thread::sleep(Duration::from_millis(5));
    }
    if drains.iter().any(|d| d.alive()) {
        let tree_reclaim = proc.cancel_within(TEARDOWN);
        let drains_done: Vec<bool> = drains
            .iter_mut()
            .map(|d| d.stop_bounded(TEARDOWN))
            .collect();
        let out = drains
            .first()
            .map(|d| d.buf.lock().unwrap().clone())
            .unwrap_or_default();
        return (
            -1,
            out,
            format!(
                "timeout reclaim: pipe holders survived root exit; tree terminated, readers stopped (tree={:?}, drains={drains_done:?})",
                tree_reclaim.map(|_| "ok").map_err(|e| e.to_string())
            ),
        );
    }
    // 正常路径：EOF 已到——统一收割（清 JoinHandle 归属）。
    let drains_done: Vec<bool> = drains
        .iter_mut()
        .map(|d| d.stop_bounded(TEARDOWN))
        .collect();
    let out = drains
        .first()
        .map(|d| d.buf.lock().unwrap().clone())
        .unwrap_or_default();
    let err = drains
        .get(1)
        .map(|d| d.buf.lock().unwrap().clone())
        .unwrap_or_default();
    if drains_done.iter().any(|d| !d) {
        return (
            -1,
            out,
            format!("reclaim incomplete after {limit:?} (drains={drains_done:?})"),
        );
    }
    (code, out, err)
}

/// 真实 CLI 入口（原四测试沿用）：程序=真实 `auto` 二进制。
fn run_bounded(
    args: &[&str],
    cwd: &Path,
    env: &[(&str, String)],
    limit: Duration,
) -> (i32, String, String) {
    run_bounded_prog(std::ffi::OsStr::new(AUTO), args, cwd, env, limit)
}

/// PLAN-816 具名测试子进程（寿命反例对端）定位：`AUTO_TE_CHILD_BIN`
/// 显式覆盖优先；默认主 workspace debug 产物（workspace 级 cargo t/裸 t
/// 全目标构建产出）。**两条路径都做存在性断言**（F-816-B-R8-01 教训：
/// env 覆盖指向缺失二进制时，诚实红灯+构建指引，而非下游 spawn 的
/// 含混 NotFound）。
fn te_child_bin() -> PathBuf {
    let p = std::env::var_os("AUTO_TE_CHILD_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            #[cfg(windows)]
            let exe = "te-child.exe";
            #[cfg(not(windows))]
            let exe = "te-child";
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/debug")
                .join(exe)
        });
    assert!(
        p.exists(),
        "te-child not found at {} — cargo build -p auto-tool-execution --bin te-child or set AUTO_TE_CHILD_BIN",
        p.display()
    );
    p
}

/// 外部写端持有者 receiver 的 RAII 守卫（F-816-B-R8-01）：任何出口
/// （含断言 panic 中止）drop 时 kill+wait 回收自有 receiver 并清理临时
/// socket/marker 文件——失败测试不遗留 fixture；正控期间不提前释放写端
/// （drop 只发生在测试作用域收尾，即原尾部清理位置）。Unix 侧另有
/// te-child 握手截止兜底自退（双保险）。
struct HolderGuard {
    child: std::process::Child,
    socket: Option<PathBuf>,
    marker: Option<PathBuf>,
}

impl HolderGuard {
    fn alive(&mut self) -> bool {
        self.child.try_wait().unwrap().is_none()
    }
}

impl Drop for HolderGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        for p in self.socket.iter().chain(self.marker.iter()) {
            let _ = std::fs::remove_file(p);
        }
    }
}

/// 外部写端持有者 fixture（F-816-B-R7-04/R8-01 永久回归共用）：
/// - Unix：`fd-receiver` receiver——bind+listen socket 后 accept 一连接，
///   recvmsg 收 SCM_RIGHTS 写 fd、写 marker 确认**真实收到**，随后持有
///   fd 存活 `hold_secs`（自带截止）；握手期以 `handshake_secs` 为整体
///   截止（无人连接/静默连接也兜底自退，exit 4）；
/// - Windows：`sleep` receiver——root 侧经 DuplicateHandle 把 stdout 写
///   句柄转入其句柄表（持有语义同；进程期即有界，无握手阶段）。
///
/// 持有者是测试拥有的独立进程，**不在 run_bounded 的 owned 树内**：
/// 杀树不给 EOF，reader 停止只能靠停止协议（Unix poll 分片/Win 取消
/// 重发）——不能用"等测试释放 writer"或 EOF 侥幸通过。守卫在 spawn 后
/// 立即建立（先于任何后续断言）并随测试作用域 drop 统一回收。返回
/// （守卫, root 的 TE_CHILD_ARG2 值）。
fn spawn_holder_receiver(
    te: &Path,
    tag: &str,
    hold_secs: u64,
    handshake_secs: u64,
) -> (HolderGuard, String) {
    #[cfg(unix)]
    {
        let socket = std::env::temp_dir().join(format!("{tag}.sock"));
        let marker = std::env::temp_dir().join(format!("{tag}.marker"));
        let child = Command::new(te)
            .env("TE_CHILD_MODE", "fd-receiver")
            .env("TE_CHILD_ARG1", socket.to_str().unwrap())
            .env("TE_CHILD_ARG2", marker.to_str().unwrap())
            .env("TE_CHILD_ARG3", hold_secs.to_string())
            .env("TE_CHILD_HANDSHAKE_SECS", handshake_secs.to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn fd receiver");
        // 守卫先于任何后续断言建立——bind 等待失败也回收 receiver。
        let guard = HolderGuard {
            child,
            socket: Some(socket.clone()),
            marker: Some(marker),
        };
        // socket 就绪（bind+listen）先于根 spawn——根的 connect 才会成功。
        let until = Instant::now() + Duration::from_secs(10);
        while !socket.exists() && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(socket.exists(), "receiver must bind socket within 10s");
        (guard, socket.to_str().unwrap().to_string())
    }
    #[cfg(windows)]
    {
        let _ = (tag, handshake_secs); // 命名/握手仅 Unix 臂使用
        let child = Command::new(te)
            .env("TE_CHILD_MODE", "sleep")
            .env("TE_CHILD_ARG1", hold_secs.to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn holder receiver");
        let pid = child.id().to_string();
        (
            HolderGuard {
                child,
                socket: None,
                marker: None,
            },
            pid,
        )
    }
}

/// F-816-B-R7-04 永久回归①：根运行至**真实超时出口**——run_bounded 返回
/// 前自有 reader 必须真停并收割（join 确认、RESIDUE 登记 0），总时长有
/// 界。holdout 根打印 marker 行→转交 stdout 写端→存活 3s（远超 280ms
/// 期限）；持有者全程在场（Unix marker=receiver 真收到写 fd，返回时刻
/// receiver 仍活）。旧实现此分支只剩 1ms 清理预算，Unix reader 一个
/// poll 分片（≤50ms）都盖不住，返回后残活 ≤100ms。
#[test]
fn timeout_exit_reaps_own_readers_before_return() {
    let te = te_child_bin();
    let (mut receiver, holder_arg) =
        spawn_holder_receiver(&te, &format!("p816-tmo-{}", std::process::id()), 8, 10);
    let start = Instant::now();
    let (code, out, err) = run_bounded_prog(
        te.as_os_str(),
        &[],
        &std::env::temp_dir(),
        &[
            ("TE_CHILD_MODE", "stdout-holdout".to_string()),
            ("TE_CHILD_ARG1", "3".to_string()),
            ("TE_CHILD_ARG2", holder_arg),
        ],
        Duration::from_millis(280),
    );
    let elapsed = start.elapsed();
    #[cfg(unix)]
    assert!(
        receiver.marker.as_ref().unwrap().exists(),
        "receiver must confirm actual write-fd transfer"
    );
    assert!(
        receiver.alive(),
        "holder must stay alive across the return (no writer release)"
    );
    assert_eq!(
        code, -1,
        "root-running timeout must classify as failure: out={out} err={err}"
    );
    assert!(
        out.contains("holdout output before sleep"),
        "root output must be preserved: {out}"
    );
    assert!(err.contains("timeout"), "classification: {err}");
    assert!(
        !err.contains("still alive"),
        "own readers must be stopped before return: {err}"
    );
    assert_eq!(
        stoppable_drain::residue_len(),
        0,
        "own readers must be reaped, not registered"
    );
    assert!(
        elapsed < Duration::from_millis(1500),
        "bounded total (limit + teardown): {elapsed:?}"
    );
    // 收尾交守卫 drop：回收 receiver + 清理临时 socket/marker——所有
    // 失败出口同样回收（F-816-B-R8-01），不再依赖尾部显式 kill。
}

/// F-816-B-R7-04 永久回归②（姊妹分支）：根早退 + 外部持有者 + 期限近
/// 尽——根退出后 150ms 排水宽限耗尽时刻压着 deadline，旧实现收尾同样
/// 从已耗尽的 deadline 挤 1ms。holdout 根转交后立即退 0（ARG1=0），
/// 200ms 期限下宽限耗尽进 holders 回收分支，reader 收割只能靠独立
/// [`TEARDOWN`] 预算真停完成（drains 全 true、RESIDUE 0）。
#[test]
fn holder_root_exit_near_deadline_reaps_readers() {
    let te = te_child_bin();
    let (mut receiver, holder_arg) =
        spawn_holder_receiver(&te, &format!("p816-hold-{}", std::process::id()), 8, 10);
    let start = Instant::now();
    let (code, out, err) = run_bounded_prog(
        te.as_os_str(),
        &[],
        &std::env::temp_dir(),
        &[
            ("TE_CHILD_MODE", "stdout-holdout".to_string()),
            ("TE_CHILD_ARG1", "0".to_string()),
            ("TE_CHILD_ARG2", holder_arg),
        ],
        Duration::from_millis(200),
    );
    let elapsed = start.elapsed();
    #[cfg(unix)]
    assert!(
        receiver.marker.as_ref().unwrap().exists(),
        "receiver must confirm actual write-fd transfer"
    );
    assert!(
        out.contains("holdout output before sleep"),
        "root output must be preserved: {out}"
    );
    // 根退出码 0 不沿报成功：持有者在场=timeout 回收分类。
    assert_eq!(
        code, -1,
        "holders must classify as timeout reclaim: out={out} err={err}"
    );
    assert!(err.contains("timeout reclaim"), "holders branch: {err}");
    assert!(
        err.contains("drains=[true, true]"),
        "readers must be reaped in teardown: {err}"
    );
    assert_eq!(
        stoppable_drain::residue_len(),
        0,
        "own readers must be reaped, not registered"
    );
    assert!(
        receiver.alive(),
        "holder must stay alive across the return (no writer release)"
    );
    assert!(
        elapsed < Duration::from_millis(1500),
        "bounded total (limit + teardown): {elapsed:?}"
    );
    // 收尾交守卫 drop（所有失败出口同样回收）。
}

/// F-816-B-R8-01 永久回归③：根在转交前失败——根指向从未 bind 的
/// socket，打印 marker 行后 connect 失败 panic 退出 101（真实子进程
/// 失败出口，无外部 wrapper）；run_bounded 如实传播根失败码，receiver
/// 未收到写 fd（marker 不存在）。Unix 侧 receiver 在握手窗口内靠
/// **握手截止自退**（exit 4 具名，本测试调短为 2s）；测试任何出口由
/// 守卫回收自有进程与临时文件——失败/正常出口均不遗留 fixture。
#[test]
fn root_failure_before_transfer_still_reclaims_receiver() {
    let te = te_child_bin();
    let (mut receiver, _holder_arg) =
        spawn_holder_receiver(&te, &format!("p816-prefail-{}", std::process::id()), 8, 2);
    let start = Instant::now();
    let bogus =
        std::env::temp_dir().join(format!("p816-prefail-{}-missing.sock", std::process::id()));
    let (code, out, err) = run_bounded_prog(
        te.as_os_str(),
        &[],
        &std::env::temp_dir(),
        &[
            ("TE_CHILD_MODE", "stdout-holdout".to_string()),
            ("TE_CHILD_ARG1", "0".to_string()),
            ("TE_CHILD_ARG2", bogus.to_str().unwrap().to_string()),
        ],
        Duration::from_secs(3),
    );
    let elapsed = start.elapsed();
    assert_eq!(
        code, 101,
        "root pre-transfer failure must propagate its exit code: out={out} err={err}"
    );
    assert!(
        out.contains("holdout output before sleep"),
        "root output must be preserved: {out}"
    );
    assert!(elapsed < Duration::from_secs(2), "bounded: {elapsed:?}");
    #[cfg(unix)]
    {
        // 转交未发生 + receiver 仍在握手窗口 → 靠握手截止自退（非测试
        // kill）：exit 4 具名、marker 不存在。
        assert!(
            !receiver.marker.as_ref().unwrap().exists(),
            "transfer must not have happened"
        );
        assert!(receiver.alive(), "receiver is in bounded handshake window");
        let until = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = receiver.child.try_wait().unwrap() {
                assert_eq!(
                    status.code(),
                    Some(4),
                    "receiver must self-exit at handshake deadline (named)"
                );
                break;
            }
            assert!(
                Instant::now() < until,
                "receiver must self-exit by handshake deadline"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }
    #[cfg(windows)]
    {
        // Windows 臂持有经 receiver 句柄表（无握手阶段）；转交失败根
        // exit 101，receiver 由守卫 drop 回收（bounded sleep 兜底）。
        assert!(receiver.alive());
    }
    // 收尾交守卫 drop。
}

/// V-04：watch 就绪横幅 + 变更触发 SFC 再生（内容身份 oracle + 单一
/// 总预算 20s + RAII 守卫含停止协议）。
#[test]
fn watch_regenerates_sfc_on_change() {
    let proj = TempProj::new("watch", "name: \"wdriver\"\nrender: \"vue\"\n", APP);
    let mut watch_spec =
        auto_tool_execution::ExecutionSpec::new([std::ffi::OsString::from(AUTO), "watch".into()]);
    watch_spec.cwd = Some(proj.path().to_path_buf());
    watch_spec.stdin = auto_tool_execution::StdioPolicy::Null;
    watch_spec.stdout = auto_tool_execution::StdioPolicy::Piped;
    watch_spec.stderr = auto_tool_execution::StdioPolicy::Null;
    watch_spec.tree_scope = auto_tool_execution::TreeScope::ProcessTree;
    let mut watch = auto_tool_execution::spawn(&watch_spec).expect("spawn watch (owned tree)");
    let (lines_tx, lines_rx) = std::sync::mpsc::channel::<String>();
    let stdout = watch.take_stdout().expect("piped stdout");
    let drain = stoppable_drain::spawn_lines(stdout, lines_tx);
    let guard = WatchGuard {
        proc: Some(watch),
        drain: Some(drain),
    };
    let total = Instant::now() + Duration::from_secs(20);
    let left = |total: Instant| -> Duration {
        total
            .saturating_duration_since(Instant::now())
            .max(Duration::from_millis(1))
    };

    let mut ready = false;
    while Instant::now() < total {
        let Ok(line) = lines_rx.recv_timeout(left(total)) else {
            break;
        };
        if line.contains("Ready. Edit an .at file") {
            ready = true;
            break;
        }
    }
    if !ready {
        panic!("watch ready banner expected within total budget (20s)");
    }
    std::fs::write(
        proj.path().join("src/front/app.at"),
        "widget App {\n    view {\n        col {\n            text \"driver v2 marker-9f3a\" {}\n        }\n    }\n}\n",
    )
    .expect("write changed app.at");
    let mut regen = false;
    let mut last = String::new();
    while Instant::now() < total {
        let Ok(line) = lines_rx.recv_timeout(left(total)) else {
            break;
        };
        last = line;
        if last.contains("SFC") && last.contains("app.at") {
            regen = true;
            break;
        }
    }
    if !regen {
        panic!("change must regenerate the SFC within total budget: last={last}");
    }
    let sfc_path = proj.path().join("gen/front/vue/src/components/App.vue");
    let sfc = std::fs::read_to_string(&sfc_path)
        .unwrap_or_else(|e| panic!("regenerated SFC must exist at {}: {e}", sfc_path.display()));
    if !sfc.contains("driver v2 marker-9f3a") {
        panic!("regenerated SFC must carry the changed text, got:\n{sfc}");
    }
    guard.release();
}

/// V-03/V-04：`auto gen` 经真实工具进程（I-TOOL 五操作链）。
#[test]
fn gen_runs_through_real_tool_process() {
    let tool = std::env::var_os("AUTO_UI_TOOL_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/debug/auto-ui-tool.exe");
            assert!(
            p.exists(),
            "auto-ui-tool not found at {} — cargo build -p auto-ui-tools or set AUTO_UI_TOOL_BIN",
            p.display()
        );
            p
        });
    let proj = TempProj::new("gen", "name: \"gdriver\"\nrender: \"vue\"\n", APP);
    let (code, out, err) = run(
        &["gen"],
        proj.path(),
        &[("AUTO_UI_TOOL_BIN", tool.to_string_lossy().to_string())],
    );
    assert_eq!(code, 0, "stdout={out} stderr={err}");
    assert!(
        proj.path().join("gen/front/vue/src/App.vue").exists()
            || proj.path().join("gen/front/vue/package.json").exists(),
        "real tool must generate the vue layout"
    );
}

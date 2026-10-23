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

use std::io::{BufRead, BufReader};
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

    pub struct Drain {
        handle: Option<std::thread::JoinHandle<()>>,
        stop: Arc<AtomicBool>,
        /// 已收字节（字节泵）；行泵不用（行直接走通道）。
        pub buf: Arc<Mutex<String>>,
        #[cfg(windows)]
        cancel_target: isize,
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
                .spawn(move || pump_unix(unix_fd, sink, flag, None))
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

/// 有界 run（F-816-B-R1-04/R4/R5-04）：设施 ProcessTree 承载——超时/
/// 根退出后 cancel_within 终止整棵 owned 树，随后 stoppable_drain 停止
/// 协议真停自有排水（Win=重发取消；Unix=poll 分片）；持有者在场=
/// timeout 回收分类（不沿根退出码报成功）；输出经共享缓冲保留；单一
/// 总预算覆盖等待/终止/回收；宽限尽未停的线程进 RESIDUE（归属不丢）。
#[allow(clippy::type_complexity)]
fn run_bounded(
    args: &[&str],
    cwd: &Path,
    env: &[(&str, String)],
    limit: Duration,
) -> (i32, String, String) {
    let mut argv: Vec<std::ffi::OsString> = vec![AUTO.into()];
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
    let left = |deadline: Instant| -> Duration {
        deadline
            .saturating_duration_since(Instant::now())
            .max(Duration::from_millis(1))
    };
    // 等待根退出（绝对到期：预算尽→终止树→真停排水→timeout 分类）。
    let code = loop {
        if Instant::now() >= deadline {
            let cancel_err = proc.cancel_within(left(deadline)).err();
            let mut stopped = true;
            for d in drains.iter_mut() {
                if !d.stop_bounded(left(deadline)) {
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
                let _ = proc.cancel_within(left(deadline));
                for d in drains.iter_mut() {
                    d.stop_bounded(left(deadline));
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
        let tree_reclaim = proc.cancel_within(left(deadline));
        let drains_done: Vec<bool> = drains
            .iter_mut()
            .map(|d| d.stop_bounded(left(deadline)))
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
        .map(|d| d.stop_bounded(left(deadline)))
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

#[allow(dead_code)]
fn _kept_imports() {
    // BufRead/BufReader 仍被 future 行泵诊断使用位；保留导入合法性。
    let _: Option<BufReader<std::io::Empty>> = None;
    let _ = std::io::empty();
}

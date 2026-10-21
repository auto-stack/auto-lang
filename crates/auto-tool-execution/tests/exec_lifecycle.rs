//! PLAN-816 T-02 正反例(AC-02 当期面):具名测试子进程 `te-child`
//! (`CARGO_BIN_EXE_te-child`)驱动的 argv/cwd/env/stdio/退出分类/
//! deadline/cancel/树退出。全部等待自带截止(观察循环有界),不依赖
//! 外部无界睡眠。

use auto_tool_execution::{
    run, spawn, ExecutionError, ExecutionSpec, ExitKind, StdioPolicy, TreeScope,
};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const TE_CHILD: &str = env!("CARGO_BIN_EXE_te-child");
const MODE: &str = "TE_CHILD_MODE";
const ARG1: &str = "TE_CHILD_ARG1";
const ARG2: &str = "TE_CHILD_ARG2";

/// te-child + 模式环境(argv 保持纯负载,协议不占 argv)。
fn child_spec(mode: &str) -> ExecutionSpec {
    let mut spec = ExecutionSpec::new([TE_CHILD]);
    spec.env.push((MODE.into(), mode.into()));
    spec
}

fn with_args(mut spec: ExecutionSpec, a1: &str, a2: Option<&str>) -> ExecutionSpec {
    spec.env.push((ARG1.into(), a1.into()));
    if let Some(a2) = a2 {
        spec.env.push((ARG2.into(), a2.into()));
    }
    spec
}

/// 每测试独立的临时目录(std-only;结尾尽力清理)。
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> TempDir {
        let dir = std::env::temp_dir().join(format!(
            "auto-tool-exec-{}-{}-{}",
            tag,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        TempDir(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// 规范化比较路径(剥 Windows 扩展前缀)。
fn canon(p: &Path) -> PathBuf {
    let c = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let s = c.to_string_lossy().to_string();
    PathBuf::from(s.strip_prefix(r"\\?\").unwrap_or(&s))
}

fn last_tick_millis(hb: &Path) -> Option<u128> {
    let content = std::fs::read_to_string(hb).ok()?;
    content
        .lines()
        .filter_map(|l| l.strip_prefix("tick "))
        .next_back()
        .and_then(|n| n.parse().ok())
}

/// 等到 heartbeat 出现首 tick(有界)。
fn wait_first_tick(hb: &Path, limit: Duration) {
    let deadline = Instant::now() + limit;
    while Instant::now() < deadline {
        if last_tick_millis(hb).is_some() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("grandchild heartbeat never started within {limit:?}");
}

// ---------------------------------------------------------------------------
// argv / cwd / env
// ---------------------------------------------------------------------------

#[test]
fn argv_exact_passthrough_spaces_unicode_empty() {
    let payload = ["--flag with spaces", "参数 含空格/Unicode", "", "plain-arg"];
    let mut spec = ExecutionSpec::new(std::iter::once(TE_CHILD).chain(payload.iter().copied()));
    spec.env.push((MODE.into(), "echo-args".into()));
    let result = run(&spec).expect("run echo-args");
    assert_eq!(
        result.kind,
        ExitKind::Success,
        "{:?} {:?}",
        result.kind,
        result.stderr_lossy()
    );
    let stdout = result.stdout_lossy().expect("captured stdout");
    let lines: Vec<&str> = stdout.trim_end_matches('\n').split('\n').collect();
    // 空参数在 te-child 里输出为空行:逐值断言(长度即身份)。
    assert_eq!(lines, payload, "argv must round-trip exactly");
}

#[test]
fn cwd_honored() {
    let tmp = TempDir::new("cwd");
    let spec = with_args(child_spec("pwd"), "", None);
    let mut spec = spec;
    spec.cwd = Some(tmp.path().to_path_buf());
    let result = run(&spec).expect("run pwd");
    assert_eq!(result.kind, ExitKind::Success);
    let printed = PathBuf::from(result.stdout_lossy().unwrap().trim().to_string());
    assert_eq!(
        canon(&printed),
        canon(tmp.path()),
        "child cwd must be the explicit spec.cwd"
    );
}

#[test]
fn env_additive_and_override() {
    // 覆盖:显式 env 值可见。
    let mut spec = with_args(child_spec("env"), "TE816_VAR", None);
    spec.env
        .push(("TE816_VAR".into(), "child-value-816".into()));
    let result = run(&spec).expect("run env override");
    assert_eq!(result.kind, ExitKind::Success);
    assert_eq!(result.stdout_lossy().unwrap().trim(), "child-value-816");

    // 增量:未声明的继承变量(PATH)仍可见,证明不是清空基线。
    let spec = with_args(child_spec("env"), "PATH", None);
    let result = run(&spec).expect("run env inherit");
    assert_eq!(result.kind, ExitKind::Success);
    assert!(
        !result.stdout_lossy().unwrap().trim().is_empty(),
        "inherited PATH must remain visible"
    );
}

// ---------------------------------------------------------------------------
// 退出 / 错误分类
// ---------------------------------------------------------------------------

#[test]
fn exit_classification_zero_and_nonzero() {
    let zero = run(&with_args(child_spec("exit"), "0", None)).expect("exit 0");
    assert_eq!(zero.kind, ExitKind::Success);

    let seven = run(&with_args(child_spec("exit"), "7", None)).expect("exit 7");
    assert_eq!(seven.kind, ExitKind::Failure(7));
}

#[test]
fn spawn_error_named_for_missing_program() {
    let spec = ExecutionSpec::new(["definitely-missing-te-child-816"]);
    let err = run(&spec).unwrap_err();
    match &err {
        ExecutionError::Spawn { program, .. } => {
            assert_eq!(program.to_string_lossy(), "definitely-missing-te-child-816");
        }
        other => panic!("expected Spawn error, got {other:?}"),
    }
    assert!(
        err.to_string().contains("definitely-missing-te-child-816"),
        "diagnostic must name the program: {err}"
    );
}

// ---------------------------------------------------------------------------
// stdio
// ---------------------------------------------------------------------------

#[test]
fn stdout_stderr_separated_capture() {
    // stderr-line 模式只写 stderr;stdout 必须为空且分开捕获。
    let spec = with_args(child_spec("stderr-line"), "err-payload-816", None);
    let result = run(&spec).expect("stderr-line");
    assert_eq!(result.kind, ExitKind::Success);
    assert_eq!(result.stdout_lossy().unwrap().trim(), "");
    assert_eq!(result.stderr_lossy().unwrap().trim(), "err-payload-816");
}

#[test]
fn big_output_no_deadlock() {
    // 512 KiB:朴素"先 wait 后读"会在管道缓冲(约 64KiB)处死锁;
    // reader 线程并发读取必须全量捕获。
    let spec = with_args(child_spec("stdout-kib"), "512", None);
    let result = run(&spec).expect("stdout-kib 512");
    assert_eq!(
        result.kind,
        ExitKind::Success,
        "{:?}",
        result.stderr_lossy()
    );
    assert_eq!(result.stdout.expect("captured").len(), 512 * 1024);
}

#[test]
fn stdin_pipe_roundtrip() {
    let payload = b"hello-stdin-816\nline-two".to_vec();
    let mut spec = with_args(child_spec("stdin-echo"), "", None);
    spec.stdin = StdioPolicy::Piped;
    spec.stdin_data = Some(payload.clone());
    let result = run(&spec).expect("stdin-echo");
    assert_eq!(result.kind, ExitKind::Success);
    assert_eq!(result.stdout.expect("captured"), payload);
}

#[test]
fn inherit_smoke_success() {
    let mut spec = with_args(child_spec("exit"), "0", None);
    spec.stdin = StdioPolicy::Inherit;
    spec.stdout = StdioPolicy::Inherit;
    spec.stderr = StdioPolicy::Inherit;
    let result = run(&spec).expect("inherit exit 0");
    assert_eq!(result.kind, ExitKind::Success);
    assert!(result.stdout.is_none() && result.stderr.is_none());
}

// ---------------------------------------------------------------------------
// deadline / cancel / 树退出
// ---------------------------------------------------------------------------

#[test]
fn deadline_timeout_kills_and_reaps() {
    let spec = with_args(child_spec("sleep"), "30", None);
    let mut spec = spec;
    spec.deadline = Some(Duration::from_millis(400));
    let result = run(&spec).expect("deadline run");
    match result.kind {
        ExitKind::Timeout { after } => {
            assert!(after >= Duration::from_millis(400), "after={after:?}");
            // cancel 收尸后 run 才返回;30s 睡眠绝不能跑完。
            assert!(
                result.elapsed < Duration::from_secs(5),
                "elapsed={:?}",
                result.elapsed
            );
        }
        other => panic!("expected Timeout, got {other:?}"),
    }
}

#[test]
fn cancel_tree_scope_kills_grandchild() {
    let tmp = TempDir::new("tree");
    let hb = tmp.path().join("hb.txt");
    // 孙进程寿命 30s(靠 cancel 树终止);先观察到它真在跑,再取消——
    // 同时确定性关闭 job assign 与孙进程 spawn 的竞态窗口。
    let mut spec = with_args(child_spec("grandchild"), "30", Some(hb.to_str().unwrap()));
    spec.tree_scope = TreeScope::ProcessTree;
    let mut proc = spawn(&spec).expect("spawn grandchild runner");

    wait_first_tick(&hb, Duration::from_secs(10));
    let m_before = last_tick_millis(&hb).expect("first tick");

    proc.cancel().expect("cancel tree");

    // kill 发出 ≠ 已停:观察 heartbeat 停止推进才判孙进程真死。
    std::thread::sleep(Duration::from_millis(600));
    let m1 = last_tick_millis(&hb).expect("ticks after cancel");
    std::thread::sleep(Duration::from_millis(600));
    let m2 = last_tick_millis(&hb).expect("ticks stable");
    assert_eq!(m1, m2, "grandchild heartbeat must stop after tree cancel");
    assert!(m1 >= m_before);
}

#[test]
fn direct_child_only_cancel_leaves_grandchild_until_own_deadline() {
    // 树语义的反证:DirectChildOnly 只杀直接子进程,孙进程按自身寿命
    // 继续推进并自停(有限 3s,无泄漏)。
    let tmp = TempDir::new("direct");
    let hb = tmp.path().join("hb.txt");
    let mut spec = with_args(child_spec("grandchild"), "3", Some(hb.to_str().unwrap()));
    spec.tree_scope = TreeScope::DirectChildOnly;
    let mut proc = spawn(&spec).expect("spawn");

    wait_first_tick(&hb, Duration::from_secs(10));

    proc.cancel().expect("cancel direct child");

    // 孙进程仍存活:heartbeat 继续推进。
    std::thread::sleep(Duration::from_millis(700));
    let m1 = last_tick_millis(&hb).expect("tick after child cancel");
    std::thread::sleep(Duration::from_millis(500));
    let m2 = last_tick_millis(&hb).expect("tick advancing");
    assert!(
        m2 > m1,
        "grandchild must survive direct-only cancel (m1={m1}, m2={m2})"
    );

    // 自停观察:孙进程 3s 寿命到后 heartbeat 不再推进(有界等待;
    // 400ms 无新 tick ≫ 100ms tick 周期,即判真停)。
    let started = Instant::now();
    let mut last = last_tick_millis(&hb).unwrap_or(0);
    let mut stopped = false;
    while started.elapsed() < Duration::from_secs(8) {
        std::thread::sleep(Duration::from_millis(400));
        let m = last_tick_millis(&hb).unwrap_or(0);
        if m == last {
            stopped = true;
            break;
        }
        last = m;
    }
    assert!(
        stopped,
        "grandchild must self-terminate within its own deadline"
    );
}

// ---------------------------------------------------------------------------
// Windows cmd shim
// ---------------------------------------------------------------------------

#[cfg(windows)]
#[test]
fn windows_cmd_shim_routes_builtin() {
    // cmd /C echo shim-ok(现行 run_command_live 形态)。
    let mut spec = ExecutionSpec::new(["echo", "shim-ok-816"]);
    spec.windows_cmd_shim = true;
    let result = run(&spec).expect("cmd /C echo");
    assert_eq!(
        result.kind,
        ExitKind::Success,
        "{:?}",
        result.stderr_lossy()
    );
    let out = result.stdout_lossy().unwrap();
    assert!(out.contains("shim-ok-816"), "stdout={out:?}");
}

// ---------------------------------------------------------------------------
// F-816-R1 复审反例的永久回归(deadline 覆盖写入/持管道后代/早派生归属/
// writer 失败具名)
// ---------------------------------------------------------------------------

/// F-816-R1-01：对端不读 stdin 时,大 stdin_data 的管道写阻塞不越过
/// deadline——250ms 截止按 Timeout 分类返回,而非等到对端 2s 自停后才以
/// BrokenPipe 冒出。
#[test]
fn deadline_covers_blocked_stdin_write() {
    let mut spec = with_args(child_spec("sleep"), "2", None);
    spec.stdin = StdioPolicy::Piped;
    spec.tree_scope = TreeScope::ProcessTree;
    spec.stdin_data = Some(vec![b'x'; 1024 * 1024]); // 远超管道缓冲
    spec.deadline = Some(Duration::from_millis(250));
    let started = Instant::now();
    let result = run(&spec).expect("run with deadline");
    let elapsed = started.elapsed();
    assert!(
        matches!(result.kind, ExitKind::Timeout { .. }),
        "{:?}",
        result.kind
    );
    assert!(
        elapsed < Duration::from_millis(1200),
        "deadline must bound the blocked write (elapsed={elapsed:?})"
    );
}

/// F-816-R1-02：根派生**继承 stdio** 的持管道后代后立即退出——run 在根
/// 真实退出分类(Success)下及时返回:owned 后代随根退出被终止,管道
/// 关闭使 reader 收到 EOF;不得等后代自停(反例实测 1818ms/Success 谎报)。
#[test]
fn root_exit_terminates_pipe_holding_descendants() {
    let dir = TempDir::new("f02-hold");
    let hb = dir.path().join("hb.log");
    let mut spec = with_args(child_spec("orphan"), "3", Some(hb.to_str().unwrap()));
    spec.tree_scope = TreeScope::ProcessTree; // 持管道后代属 owned 树
    spec.deadline = Some(Duration::from_secs(5)); // 根秒退,不会触超时
    let started = Instant::now();
    let result = run(&spec).expect("run");
    let elapsed = started.elapsed();
    assert_eq!(result.kind, ExitKind::Success, "{:?}", result.kind);
    assert!(
        elapsed < Duration::from_millis(1500),
        "root exit must bound the run (elapsed={elapsed:?})"
    );
    // 持管道后代确已被 owned 树终止:heartbeat 停止推进。
    wait_first_tick(&hb, Duration::from_secs(2));
    let stopped_at = std::fs::metadata(&hb).map(|m| m.len()).unwrap_or(0);
    std::thread::sleep(Duration::from_millis(600));
    let after = std::fs::metadata(&hb).map(|m| m.len()).unwrap_or(0);
    assert_eq!(stopped_at, after, "pipe-holding descendant must be dead");
}

/// F-816-R1-04:spawn 返回前即已存在的后代属于 owned 树——挂起入 Job
/// 机制下,根的任何用户代码(含派生)都在归属建立之后运行,早派生的
/// 后代不可能逃出 Job;cancel 后 heartbeat 停止(确定性,不靠等待时长)。
#[test]
fn early_spawned_descendant_belongs_to_owned_tree() {
    let dir = TempDir::new("f04-early");
    let hb = dir.path().join("hb.log");
    let mut spec = with_args(child_spec("orphan"), "8", Some(hb.to_str().unwrap()));
    spec.tree_scope = TreeScope::ProcessTree; // 早派生后代必须入 owned 树
    let mut proc = spawn(&spec).expect("spawn");
    wait_first_tick(&hb, Duration::from_secs(2)); // 后代已存在
    proc.cancel().expect("cancel owned tree");
    let stopped_at = std::fs::metadata(&hb).map(|m| m.len()).unwrap_or(0);
    std::thread::sleep(Duration::from_millis(600));
    let after = std::fs::metadata(&hb).map(|m| m.len()).unwrap_or(0);
    assert_eq!(stopped_at, after, "early descendant must die with the tree");
}

/// F-816-R1-06(正常路径 writer 面):子进程不读 stdin 即退出,大
/// stdin_data 的写失败具名传播(write stdin),不以 Success/空输出掩盖。
#[test]
fn stdin_writer_failure_is_named() {
    let mut spec = with_args(child_spec("exit"), "0", None);
    spec.stdin = StdioPolicy::Piped;
    spec.stdin_data = Some(vec![b'x'; 1024 * 1024]); // 子进程即退,写必断管
    let err = run(&spec).expect_err("writer must fail on broken pipe");
    match &err {
        ExecutionError::Io { stage, .. } => assert_eq!(*stage, "write stdin"),
        other => panic!("expected named write-stdin failure, got {other:?}"),
    }
}

/// F-816-R5 写端真停（Windows 实机,生产路径）：根存活 +
/// 非拥有持读者（stdin-holder 后代继承 stdin 且不读）——1MiB
/// stdin_data 超缓冲阻塞写;250ms deadline 后 run 必须以 Timeout 分类
/// 有界返回（写线程经 CancelIoEx 取消真停,kind=Stopped 部分写;
/// 非owned 后代保留,heartbeat 继续推进）。
#[test]
fn deadline_covers_blocked_stdin_with_unowned_holder() {
    let dir = TempDir::new("r5-writer");
    let hb = dir.path().join("hb.log");
    let mut spec = with_args(child_spec("stdin-holder"), "5", Some(hb.to_str().unwrap()));
    spec.stdin = StdioPolicy::Piped;
    spec.tree_scope = TreeScope::DirectChildOnly; // 非拥有后代保留(scope 合同)
    spec.stdin_data = Some(vec![b'x'; 1024 * 1024]);
    spec.deadline = Some(Duration::from_millis(250));
    let started = Instant::now();
    let result = run(&spec).expect("run with deadline");
    let elapsed = started.elapsed();
    assert!(
        matches!(result.kind, ExitKind::Timeout { .. }),
        "{:?}",
        result.kind
    );
    assert!(
        elapsed < Duration::from_millis(1500),
        "writer must truly stop via cancel (elapsed={elapsed:?})"
    );
}

/// F-816-R9 负控（真实消费入口）：run 的准入失败家族——stdin_data 存在
/// 而 stdin 未管道化 → validate 即具名拒绝（早于任何线程/进程创建）。
/// 创建/复制失败分支的同等出口由准入顺序修复保证（R9 方向②：先准入
/// 后建线程——失败时无 reader/writer 存在,宿主由 OwnedProcess::drop
/// 回收）;R9 admission 回放（分配边界注入）验证线程 4→4。
#[test]
fn run_admission_failure_is_named_and_threadless() {
    let mut spec = with_args(child_spec("sleep"), "2", None);
    spec.stdin = StdioPolicy::Inherit;
    spec.stdin_data = Some(vec![b'x'; 16]);
    spec.stdout = StdioPolicy::Null;
    spec.stderr = StdioPolicy::Null;
    let started = Instant::now();
    let err = run(&spec).expect_err("admission must reject");
    assert!(
        matches!(
            err,
            ExecutionError::InvalidSpec(ref m) if m.contains("stdin_data requires stdin")
        ),
        "{err:?}"
    );
    assert!(started.elapsed() < Duration::from_secs(2));
}

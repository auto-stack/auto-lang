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

#[test]
fn prepared_command_keeps_environment_clear_argv_and_cwd() {
    use auto_tool_execution::run_command_capture;
    let temp = TempDir::new("prepared-command");
    let mut command = std::process::Command::new(TE_CHILD);
    command
        .env_clear()
        .env(MODE, "probe")
        .env(ARG1, "PREPARED_COMMAND_MARKER")
        .env("PREPARED_COMMAND_MARKER", "actual-value")
        .current_dir(temp.path());
    let result = run_command_capture(
        command,
        TreeScope::ProcessTree,
        Some(Duration::from_secs(5)),
    )
    .unwrap();
    assert_eq!(result.kind, ExitKind::Success);
    let stderr = result.stderr_lossy().unwrap();
    assert!(stderr.contains("PREPARED_COMMAND_MARKER=actual-value"));
    assert!(stderr.contains(temp.path().to_str().unwrap()));

    let key = "AUTO_TOOL_EXEC_PREPARED_PARENT_ONLY";
    let previous = std::env::var_os(key);
    std::env::set_var(key, "must-not-inherit");
    let mut command = std::process::Command::new(TE_CHILD);
    command.env_clear().env(MODE, "env").env(ARG1, key);
    let result = run_command_capture(
        command,
        TreeScope::ProcessTree,
        Some(Duration::from_secs(5)),
    );
    match previous {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
    assert_eq!(
        result.unwrap().kind,
        ExitKind::Failure(3),
        "env_clear must survive the adapter"
    );

    let mut command = std::process::Command::new(TE_CHILD);
    command
        .env(MODE, "echo-args")
        .args(["with spaces", "", "literal;$value"]);
    let result = run_command_capture(
        command,
        TreeScope::ProcessTree,
        Some(Duration::from_secs(5)),
    )
    .unwrap();
    assert_eq!(result.kind, ExitKind::Success);
    assert_eq!(
        result.stdout_lossy().unwrap(),
        "with spaces\n\nliteral;$value\n"
    );
}

#[test]
fn prepared_command_timeout_keeps_partial_capture() {
    let mut command = std::process::Command::new(TE_CHILD);
    command
        .env(MODE, "stdout-kib")
        .env(ARG1, "1")
        .env(ARG2, "30");
    let result = auto_tool_execution::run_command_capture(
        command,
        TreeScope::ProcessTree,
        Some(Duration::from_millis(500)),
    )
    .unwrap();
    assert!(matches!(result.kind, ExitKind::Timeout { .. }));
    assert_eq!(result.stdout.as_ref().unwrap(), &vec![b'x'; 1024]);
}

#[test]
fn prepared_command_preserves_invalid_utf8_as_raw_capture() {
    let mut command = std::process::Command::new(TE_CHILD);
    command.env(MODE, "stdout-invalid-utf8");
    let result = auto_tool_execution::run_command_capture(
        command,
        TreeScope::ProcessTree,
        Some(Duration::from_secs(5)),
    )
    .unwrap();
    assert_eq!(result.kind, ExitKind::Success);
    assert_eq!(
        result.stdout.as_deref(),
        Some(b"{\"value\":\"\xff\"}".as_slice())
    );
}

#[test]
fn prepared_command_reaps_the_exited_parents_descendants() {
    let temp = TempDir::new("prepared-tree");
    let heartbeat = temp.path().join("heartbeat");
    let mut command = std::process::Command::new(TE_CHILD);
    command
        .env(MODE, "grandchild")
        .env(ARG1, "1")
        .env(ARG2, &heartbeat);
    let result = auto_tool_execution::run_command_capture(
        command,
        TreeScope::ProcessTree,
        Some(Duration::from_secs(5)),
    )
    .unwrap();
    assert_eq!(result.kind, ExitKind::Success);
    let before = last_tick_millis(&heartbeat).expect("the real descendant ran");
    std::thread::sleep(Duration::from_millis(350));
    assert_eq!(
        last_tick_millis(&heartbeat),
        Some(before),
        "no owned descendant after return"
    );
}

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

/// PLAN-815 R4-01（F-815-T05-R4-01）：`env_remove` 先于 `env` 应用——
/// 宿主进程环境里的陈旧同名变量不漏进子进程（"每个子进程只获得本次
/// 调用的上下文"隔离合同）。正控=显式对在场；负控=继承值被移除。
#[test]
fn env_remove_strips_inherited_key_before_env_adds() {
    const PROBE: &str = "TE816_REMOVE_PROBE";
    // SAFETY: nextest 单测单进程;测试尾恢复。
    unsafe {
        std::env::set_var(PROBE, "stale-host-value");
    }

    // 正控:显式 env 对(移除同名继承值后)在场。
    let mut spec = with_args(child_spec("env"), PROBE, None);
    spec.env_remove.push(PROBE.into());
    spec.env.push((PROBE.into(), "per-request".into()));
    let result = run(&spec).expect("run env remove+add");
    assert_eq!(result.kind, ExitKind::Success);
    assert_eq!(result.stdout_lossy().unwrap().trim(), "per-request");

    // 负控:仅移除——子进程看不到宿主继承值(te-child env 模式缺失=exit 3)。
    let mut spec = with_args(child_spec("env"), PROBE, None);
    spec.env_remove.push(PROBE.into());
    let result = run(&spec).expect("run env remove-only");
    assert!(
        matches!(result.kind, ExitKind::Failure(3)),
        "inherited value must be stripped, got {:?}",
        result.kind
    );

    // SAFETY: 恢复测试进程环境。
    unsafe {
        std::env::remove_var(PROBE);
    }
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

// ---- PLAN-816 T-06：spawn_detached（daemon 分离）与 terminate 原语 ----

/// 分离 spawn 后子进程**不被本端持有**：句柄即弃，心跳子进程在无任何
/// 归属的情况下继续推进（daemon 语义；bounded 自退出 2s）。
#[test]
fn detached_child_outlives_spawn_caller_handle() {
    let scratch = TempDir::new("detach");
    let beat = scratch.path().join("beat.txt");
    let spec = with_args(child_spec("heartbeat"), "2", Some(&beat.to_string_lossy()));
    #[cfg(windows)]
    let mut spec = spec;
    #[cfg(windows)]
    {
        spec.windows_creation_flags = 0x0800_0000; // CREATE_NO_WINDOW（serve 形态）
    }
    let pid = auto_tool_execution::spawn_detached(&spec).expect("detached spawn");
    assert!(pid > 0);
    // 观察窗口：本端已无句柄；心跳推进=子进程自立存活。
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut ticks = 0u32;
    while Instant::now() < deadline {
        if let Ok(content) = std::fs::read_to_string(&beat) {
            ticks = content.lines().filter(|l| !l.is_empty()).count() as u32;
            if ticks >= 3 {
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(
        ticks >= 3,
        "detached child must keep living (ticks={ticks})"
    );
}

/// 非零 creation flags 在非 Windows 拒绝（InvalidSpec；Windows 本机恒可
/// 配置，负控走 cfg 分支编译面）。
#[cfg(not(windows))]
#[test]
fn creation_flags_are_windows_only() {
    let mut spec = with_args(child_spec("exit"), "0", None);
    spec.windows_creation_flags = 1;
    let err = auto_tool_execution::spawn_detached(&spec).unwrap_err();
    assert!(matches!(err, ExecutionError::InvalidSpec(_)), "{err:?}");
}

/// 终止原语：安装后 OS 真实信号（Windows 控制台 CTRL_C / Unix SIGINT）
/// 投递到子进程组 → 子进程观察到标志干净退出码 42（不是被强杀）。
#[test]
fn terminate_flag_observes_real_os_signal() {
    let mut cmd = std::process::Command::new(TE_CHILD);
    cmd.env(MODE, "terminate-wait");
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x200;
        cmd.creation_flags(CREATE_NEW_PROCESS_GROUP);
    }
    let mut child = cmd.spawn().expect("spawn terminate-wait child");
    // 稍等安装完成（信号早到=子进程未装处理器=默认终止,测试红即线索）。
    std::thread::sleep(Duration::from_millis(300));
    #[cfg(windows)]
    {
        const CTRL_C_EVENT: u32 = 0;
        extern "system" {
            fn GenerateConsoleCtrlEvent(event: u32, pgid: u32) -> i32;
        }
        let sent = unsafe { GenerateConsoleCtrlEvent(CTRL_C_EVENT, child.id()) };
        assert_ne!(sent, 0, "GenerateConsoleCtrlEvent failed");
    }
    #[cfg(unix)]
    {
        extern "C" {
            fn kill(pid: i32, sig: i32) -> i32;
        }
        let sent = unsafe {
            kill(child.id() as i32, 2 /* SIGINT */)
        };
        assert_eq!(sent, 0, "kill(SIGINT) failed");
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match child.try_wait().expect("try_wait") {
            Some(st) => {
                assert_eq!(st.code(), Some(42), "clean observe-and-exit, not a kill");
                break;
            }
            None => {
                assert!(Instant::now() < deadline, "child did not exit in 5s");
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

/// 终止原语程序化面：request 置位 requested 观察（同一标志;嵌入方路径）。
#[test]
fn terminate_programmatic_request_sets_flag() {
    assert!(!auto_tool_execution::terminate::requested());
    auto_tool_execution::terminate::request();
    assert!(auto_tool_execution::terminate::requested());
}

/// F-816-B-R8-01 永久回归（fd-receiver 握手截止）：无人连接时 receiver
/// 不再无限阻塞在 accept——握手截止（TE_CHILD_HANDSHAKE_SECS=1）到期
/// **自退** exit 4 具名，marker 不存在（未收到写 fd）。
#[cfg(unix)]
#[test]
fn fd_receiver_self_exits_without_peer() {
    let dir = TempDir::new("r801-npeer");
    let socket = dir.path().join("h.sock");
    let marker = dir.path().join("h.marker");
    let mut child = std::process::Command::new(TE_CHILD)
        .env(MODE, "fd-receiver")
        .env(ARG1, socket.to_str().unwrap())
        .env(ARG2, marker.to_str().unwrap())
        .env("TE_CHILD_ARG3", "1")
        .env("TE_CHILD_HANDSHAKE_SECS", "1")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn fd-receiver");
    let ready_by = Instant::now() + Duration::from_secs(2);
    while !socket.exists() && Instant::now() < ready_by {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(socket.exists(), "receiver must bind socket");
    // socket 就绪后无人连接——握手截止自退（非测试 kill）。
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        match child.try_wait().expect("try_wait") {
            Some(st) => {
                assert_eq!(st.code(), Some(4), "handshake-deadline self-exit is named");
                break;
            }
            None => {
                assert!(Instant::now() < deadline, "no-peer receiver must self-exit");
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
    assert!(!marker.exists(), "no write fd was ever received");
}

/// F-816-B-R8-01 永久回归（静默连接）：对端 connect 后不发送任何字节
/// ——recvmsg 同受握手截止约束，到期具名自退 exit 4，marker 不存在。
/// 连接全程保持打开（真实静默语义，非 EOF 竞态）。
#[cfg(unix)]
#[test]
fn fd_receiver_self_exits_on_silent_peer() {
    let dir = TempDir::new("r801-speer");
    let socket = dir.path().join("h.sock");
    let marker = dir.path().join("h.marker");
    let mut child = std::process::Command::new(TE_CHILD)
        .env(MODE, "fd-receiver")
        .env(ARG1, socket.to_str().unwrap())
        .env(ARG2, marker.to_str().unwrap())
        .env("TE_CHILD_ARG3", "1")
        .env("TE_CHILD_HANDSHAKE_SECS", "1")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn fd-receiver");
    let ready_by = Instant::now() + Duration::from_secs(2);
    while !socket.exists() && Instant::now() < ready_by {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(socket.exists(), "receiver must bind socket");
    let peer = std::os::unix::net::UnixStream::connect(&socket).expect("connect silent peer");
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        match child.try_wait().expect("try_wait") {
            Some(st) => {
                assert_eq!(st.code(), Some(4), "handshake-deadline self-exit is named");
                break;
            }
            None => {
                assert!(
                    Instant::now() < deadline,
                    "silent-peer receiver must self-exit"
                );
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
    drop(peer);
    assert!(!marker.exists(), "silent peer sent nothing");
}

/// F-816-B-R9-01 永久回归（signal_group 组内活后代受控送达）：
/// interrupt() 的 SIGINT 必须到达 owned 组内活后代——terminate-wait 后代
/// 的受控 handler 观察到信号并干净退 42（非默认致死、非根独吞）；根同
/// 在组内靠自身 handler 存活传播后代退出码。旧实现（正 PGID）只命中
/// 根：根 handler 吞掉信号、后代收不到，10s 截止退 7——42/7 双态区分
/// 送达与未送达。
#[cfg(unix)]
#[test]
fn interrupt_signal_reaches_group_descendants() {
    let dir = TempDir::new("r901-sigtree");
    let marker = dir.path().join("ready.marker");
    let mut spec = with_args(child_spec("signal-tree"), marker.to_str().unwrap(), None);
    spec.tree_scope = TreeScope::ProcessTree;
    let mut proc = spawn(&spec).expect("spawn signal-tree (owned group)");
    // 双端 handler 安装就绪（根在后代 handler 就绪后才写 marker——
    // 消除安装/送达竞态，无固定睡眠）。
    let ready_by = Instant::now() + Duration::from_secs(10);
    while !marker.exists() && Instant::now() < ready_by {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(marker.exists(), "both handlers must install within 10s");
    proc.interrupt().expect("interrupt owned group");
    // 根退出码=后代退出码：42=组 SIGINT 送达受控 handler；7=未送达
    //（后代 10s 截止自守）——有界等待覆盖两态。
    let deadline = Instant::now() + Duration::from_secs(13);
    loop {
        match proc
            .wait_deadline(Duration::from_millis(200))
            .expect("wait")
        {
            Some(status) => {
                assert_eq!(
                    status.code(),
                    Some(42),
                    "group SIGINT must reach the descendant handler"
                );
                break;
            }
            None => {
                assert!(Instant::now() < deadline, "signal tree did not exit in 13s");
            }
        }
    }
}

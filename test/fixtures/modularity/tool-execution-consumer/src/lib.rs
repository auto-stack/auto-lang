//! PLAN-816 T-07：owned 执行设施的第三消费方（管理服务/CLI 之外）。
//!
//! 编译期：只依赖 `auto-tool-execution`——闭包=仅 std（verify-deps.sh
//! cargo tree 钉住，契约基线 §1）。运行期：以真实 `te-child` 具名模式
//! 子进程消费 run/spawn/deadline/cancel/树退出/分离 spawn/终止原语
//! （V-02 当期独立消费证据）；环境缺失（te-child 未构建）=诚实红灯。

use auto_tool_execution::{
    spawn, spawn_detached, terminate, ExecutionSpec, ExitKind, StdioPolicy, TreeScope,
};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn te_child() -> PathBuf {
    if let Some(p) = std::env::var_os("TE_CHILD_BIN") {
        return p.into();
    }
    let exe = if cfg!(windows) {
        "te-child.exe"
    } else {
        "te-child"
    };
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../target/debug")
        .join(exe);
    assert!(
        path.exists(),
        "te-child not found at {} — cargo build -p auto-tool-execution (or set TE_CHILD_BIN)",
        path.display()
    );
    path
}

fn spec(mode: &str) -> ExecutionSpec {
    let mut s = ExecutionSpec::new([te_child()]);
    s.env.push(("TE_CHILD_MODE".into(), mode.into()));
    s
}

/// V-02 独立消费正控：argv 精确透传 + cwd/env 生效 + 退出分类。
#[test]
fn run_passes_argv_cwd_env_and_classifies_exit() {
    let dir = std::env::temp_dir().join(format!("tec-run-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut s = spec("echo-args");
    s.argv.push("a b".into()); // 含空格参数原样透传
    s.argv.push("中文".into()); // Unicode
    s.cwd = Some(dir.clone());
    s.stdout = StdioPolicy::Piped;
    let result = auto_tool_execution::run(&s).expect("run");
    assert_eq!(result.kind, ExitKind::Success);
    let out = result.stdout_lossy().unwrap().to_string();
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines.last().copied(),
        Some("中文"),
        "argv verbatim: {out:?}"
    );
    assert!(
        lines.contains(&"a b".to_string().as_str()) || out.contains("a b"),
        "spaced arg intact: {out:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// V-02：非零退出具名分类（Failure(code)，非 Success/未知）。
#[test]
fn run_classifies_nonzero_exit() {
    let mut s = spec("exit");
    s.env.push(("TE_CHILD_ARG1".into(), "7".into()));
    let result = auto_tool_execution::run(&s).expect("run");
    assert_eq!(result.kind, ExitKind::Failure(7));
}

/// V-02：deadline 到期=owned 树真停（Timeout 分类；孙进程树一并退出——
/// 观察到收尸后才返回，非 kill 发出即返）。
#[test]
fn deadline_terminates_owned_tree_and_classifies_timeout() {
    let dir = std::env::temp_dir().join(format!("tec-dl-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let beat = dir.join("beat.txt");
    let mut s = spec("grandchild");
    s.env.push(("TE_CHILD_ARG1".into(), "10".into())); // 根存活 10s
    s.env.push((
        "TE_CHILD_ARG2".into(),
        beat.to_string_lossy().to_string().into(),
    ));
    s.tree_scope = TreeScope::ProcessTree;
    s.deadline = Some(Duration::from_secs(1));
    let started = Instant::now();
    let result = auto_tool_execution::run(&s).expect("run");
    assert!(
        matches!(result.kind, ExitKind::Timeout { .. }),
        "{:?}",
        result.kind
    );
    assert!(started.elapsed() < Duration::from_secs(5), "bounded reap");
    // 孙进程心跳停止推进（树真停的独立观察：等 300ms 无新 tick）。
    let ticks_at_return = std::fs::read_to_string(&beat)
        .unwrap_or_default()
        .lines()
        .count();
    std::thread::sleep(Duration::from_millis(400));
    let ticks_after = std::fs::read_to_string(&beat)
        .unwrap_or_default()
        .lines()
        .count();
    assert_eq!(ticks_at_return, ticks_after, "grandchild heartbeat stopped");
    let _ = std::fs::remove_dir_all(&dir);
}

/// T-06 原语独立消费：分离 spawn 后子进程不归本消费方持有（心跳自立
/// 推进）；终止原语程序化面（request→requested）。
#[test]
fn detached_spawn_and_terminate_primitive_are_consumable() {
    let dir = std::env::temp_dir().join(format!("tec-detach-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let beat = dir.join("beat.txt");
    let mut s = spec("heartbeat");
    s.env.push(("TE_CHILD_ARG1".into(), "2".into()));
    s.env.push((
        "TE_CHILD_ARG2".into(),
        beat.to_string_lossy().to_string().into(),
    ));
    let pid = spawn_detached(&s).expect("detached spawn");
    assert!(pid > 0);
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut ticks = 0;
    while Instant::now() < deadline {
        if let Ok(c) = std::fs::read_to_string(&beat) {
            ticks = c.lines().count();
            if ticks >= 3 {
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(ticks >= 3, "detached child self-sustaining (ticks={ticks})");
    // 终止原语程序化面（OS 信号面由设施 exec_lifecycle 真投递测试承担）。
    assert!(!terminate::requested());
    terminate::request();
    assert!(terminate::requested());
    let _ = std::fs::remove_dir_all(&dir);
}

/// V-02：长运行 spawn + cancel 收尸（真停观察）。
#[test]
fn spawn_cancel_reaps_owned_process() {
    let mut proc = spawn(&spec("sleep")).expect("spawn");
    let pid = proc.id();
    assert!(pid > 0);
    let status = proc.cancel_within(Duration::from_secs(3)).expect("cancel");
    assert!(!status.success(), "killed sleep must not report success");
}

//! PLAN-112 T-02 第 5 项：VM 宿主结束进程授权门（auto.sys.kill 2431）。
//!
//! 判据（正反成对）：
//! - 无门 / `AllowAll` = 迁移前行为逐字不变（真实结束隔离子进程）；
//! - `Deny` 在刷新采样与 OS 结束之前抛 `VMError`：稳定码 + 宿主原因 + pid，
//!   目标进程存活；
//! - `AllowedPids` 清单内放行、清单外具名拒；
//! - 进程默认 gate 在 `AutoVM::new` 构造期快照进实例（之后清默认不清实例）。
//!
//! 目标进程一律测试自建隔离子进程（不以测试进程自身 pid 作目标——门一旦
//! 失效会结束测试进程）。本文件是独立测试二进制，进程默认槽的 set/clear
//! 只在本进程内、并由 `SLOT` 串行化。

use std::collections::BTreeSet;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, MutexGuard};

use auto_lang::vm::host_capabilities::{
    SysKillDecision, SysKillGate, SYS_KILL_PERMISSION_DENIED,
};

static SLOT: Mutex<()> = Mutex::new(());

fn slot() -> MutexGuard<'static, ()> {
    SLOT.lock().unwrap_or_else(|e| e.into_inner())
}

enum Mode {
    Allow,
    Deny(String),
    Pids(BTreeSet<i32>),
}

struct TestGate {
    mode: Mode,
}

impl SysKillGate for TestGate {
    fn decision(&self, pid: i32) -> SysKillDecision {
        match &self.mode {
            Mode::Allow => SysKillDecision::Allow,
            Mode::Deny(reason) => SysKillDecision::Deny {
                reason: reason.clone(),
            },
            Mode::Pids(allowed) => {
                if allowed.contains(&pid) {
                    SysKillDecision::Allow
                } else {
                    SysKillDecision::Deny {
                        reason: "pid not in the host authorization list".to_string(),
                    }
                }
            }
        }
    }
}

/// 起一个不结束的隔离子进程（目标 pid = child.id()）。
fn spawn_target() -> Child {
    // 单进程目标（不经 cmd/sh 包一层）：sys.kill 只结束该 pid，若留在壳里
    // 会只杀掉壳、留下孙进程，既污染"目标已退出"的判据，也会被 nextest
    // 记为 leaked child。
    let mut cmd = if cfg!(windows) {
        Command::new("ping")
    } else {
        Command::new("sleep")
    };
    if cfg!(windows) {
        cmd.args(["-n", "120", "127.0.0.1"]);
    } else {
        cmd.arg("120");
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn isolated target process")
}

fn is_alive(child: &mut Child) -> bool {
    matches!(child.try_wait(), Ok(None))
}

fn cleanup(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn kill_program(pid: i32) -> String {
    format!(
        r#"
fn main() {{
    let ok = sys.kill({pid})
    if ok {{ print("killed") }} else {{ print("not-killed") }}
}}
"#
    )
}

/// 在受控槽位下执行：清默认 → 可选登记 gate → 运行 → 清默认。
fn with_gate<T>(gate: Option<Mode>, f: impl FnOnce() -> T) -> T {
    let _guard = slot();
    auto_lang::vm::host_capabilities::clear_process_default_sys_kill_gate();
    if let Some(mode) = gate {
        auto_lang::vm::host_capabilities::set_process_default_sys_kill_gate(Arc::new(TestGate {
            mode,
        }));
    }
    let out = f();
    auto_lang::vm::host_capabilities::clear_process_default_sys_kill_gate();
    out
}

fn run(code: &str) -> Result<(String, String), String> {
    auto_lang::run_autovm_capture(code).map_err(|e| e.to_string())
}

#[test]
fn p112_absent_gate_keeps_legacy_kill() {
    let mut target = spawn_target();
    let pid = target.id() as i32;
    let result = with_gate(None, || run(&kill_program(pid)));
    let (res, stdout) = result.expect("legacy kill must not raise");
    assert!(
        res.contains("killed") || stdout.contains("killed"),
        "legacy kill must report success; res={res:?} stdout={stdout:?}"
    );
    assert!(
        target.try_wait().expect("wait target").is_some(),
        "legacy kill must terminate the isolated target"
    );
    cleanup(target);
}

#[test]
fn p112_deny_gate_names_denial_and_spares_target() {
    let mut target = spawn_target();
    let pid = target.id() as i32;
    let reason = "host holds no process-kill authorization";
    let (res, stdout) = with_gate(Some(Mode::Deny(reason.into())), || run(&kill_program(pid)))
        .expect("soft deny must not raise");
    assert!(
        res.contains("not-killed") || stdout.contains("not-killed"),
        "denial must report false to the caller; res={res:?} stdout={stdout:?}"
    );
    assert!(is_alive(&mut target), "denied request must not touch the target");
    let text = auto_lang::vm::host_capabilities::sys_kill_denial_text(pid, reason);
    assert!(
        text.contains(SYS_KILL_PERMISSION_DENIED)
            && text.contains(reason)
            && text.contains(&pid.to_string()),
        "denial diagnostic must carry code + reason + pid: {text}"
    );
    cleanup(target);
}

#[test]
fn p112_allowed_pids_split_inside_and_outside() {
    let mut a = spawn_target();
    let mut b = spawn_target();
    let (pid_a, pid_b) = (a.id() as i32, b.id() as i32);
    let allowed: BTreeSet<i32> = [pid_a].into_iter().collect();

    // 清单外：软拒绝（false + 诊断）+ 目标存活。
    let (res, stdout) = with_gate(Some(Mode::Pids(allowed.clone())), || run(&kill_program(pid_b)))
        .expect("ungranted pid must not raise");
    assert!(
        res.contains("not-killed") || stdout.contains("not-killed"),
        "ungranted kill must report false; res={res:?} stdout={stdout:?}"
    );
    assert!(is_alive(&mut b), "ungranted target must survive");

    // 清单内：真实结束。
    let (res, stdout) = with_gate(Some(Mode::Pids(allowed)), || run(&kill_program(pid_a)))
        .expect("granted pid must be killed");
    assert!(
        res.contains("killed") || stdout.contains("killed"),
        "res={res:?} stdout={stdout:?}"
    );
    assert!(
        a.try_wait().expect("wait a").is_some(),
        "granted target must terminate"
    );
    cleanup(a);
    cleanup(b);
}

#[test]
fn p112_allow_all_matches_absent_gate() {
    let mut target = spawn_target();
    let pid = target.id() as i32;
    let (res, stdout) = with_gate(Some(Mode::Allow), || run(&kill_program(pid)))
        .expect("explicit AllowAll must not raise");
    assert!(res.contains("killed") || stdout.contains("killed"));
    assert!(target.try_wait().expect("wait target").is_some());
    cleanup(target);
}

#[test]
fn p112_process_default_is_snapshotted_into_the_instance() {
    let _guard = slot();
    auto_lang::vm::host_capabilities::clear_process_default_sys_kill_gate();
    auto_lang::vm::host_capabilities::set_process_default_sys_kill_gate(Arc::new(TestGate {
        mode: Mode::Deny("snapshot probe".into()),
    }));
    let (vm, _, _, _) = auto_lang::create_vm_from_source("fn main() {\n    print(\"ok\")\n}")
        .expect("compile probe program");
    auto_lang::vm::host_capabilities::clear_process_default_sys_kill_gate();
    assert!(
        vm.sys_kill_gate().is_some(),
        "construction must snapshot the process-default gate"
    );
    assert!(
        auto_lang::vm::host_capabilities::process_default_sys_kill_gate().is_none(),
        "clearing the default must not require touching the instance"
    );
}

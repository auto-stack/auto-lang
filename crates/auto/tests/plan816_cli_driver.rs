//! PLAN-816 T-07：真实 CLI 进程 driver（V-01/V-04 的可自动化切片）。
//!
//! 以 `CARGO_BIN_EXE_auto` 驱动**真实二进制**（真 CLI→语言/管理/服务/
//! 工具链路，非单元层模拟）：
//! - V-01 裸脚本/argv 透传（`--` 后强制透传，Plan 524 契约）；
//! - V-04 serve/req 具名管道往返（daemon 协议真实入口）；
//! - V-04 watch 变更触发 SFC 再生（HMR 前置）+ 终止退出；
//! - V-04 gen 经真实工具进程（I-TOOL 五操作链，AUTO_UI_TOOL_BIN 指向
//!   主 workspace 产物；缺工具=诚实红灯）。
//! 重面（service vm/rust 真链冷启动、Core profile 独立构建闭包）由
//! attachments/816/t06-smoke-results.json（真链冒烟）与 T-05 双形态
//! cargo tree 证据承载，不在默认档重复跑。

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
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

/// watch 子进程 RAII 守卫（F-816-B-R1-04/R3 残项）：经设施 ProcessTree
/// 承载——任何出口（含 panic 与 unwrap 失败）都先 `cancel_within` 终止
/// **整棵 owned 树**（持管道后代一并死→EOF→reader 真停），再有界 join
/// reader 线程；守卫不再无限期等待超出预算的后代。
struct WatchGuard {
    proc: Option<auto_tool_execution::OwnedProcess>,
    pump: Option<std::thread::JoinHandle<()>>,
}

impl WatchGuard {
    fn release(mut self) {
        self.stop_bounded();
    }
    fn stop_bounded(&mut self) {
        if let Some(mut proc) = self.proc.take() {
            let _ = proc.cancel_within(Duration::from_secs(5));
        }
        if let Some(p) = self.pump.take() {
            let deadline = Instant::now() + Duration::from_secs(5);
            while !p.is_finished() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(5));
            }
            if p.is_finished() {
                let _ = p.join();
            }
            // 未完：树已终止、EOF 已确定——残余只可能是非拥有持有者，
            // 不再无限期等待（R3 残项 B 的 23s 后代即在此被切断）。
        }
    }
}

impl Drop for WatchGuard {
    fn drop(&mut self) {
        self.stop_bounded();
    }
}

const APP: &str = "widget App {\n    view {\n        col {\n            text \"driver\" {}\n        }\n    }\n}\n";

/// V-01：裸脚本执行 + `--` 后 argv 强制透传（脚本读到 3 个参数；旧
/// Plan 524 契约在薄 CLI 形态下保持）。
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
    // 返回值行是 stdout 的最后一个非空行（前面是 CLI 的 Running 横幅）；
    // process.args() 含脚本路径自身（裸跑=1，Plan 524 契约），故 `--` 后
    // 透传 3 参 = 4。
    let scalar = out
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .and_then(|l| l.trim().trim_matches('"').parse::<usize>().ok())
        .unwrap_or_else(|| panic!("argv length scalar expected, got {out:?}"));
    assert_eq!(scalar, 4, "script path + three passthrough args");
    let _ = std::fs::remove_dir_all(&dir);
}

/// V-04：serve（前台 daemon）+ req 具名管道往返——新建会话后在列表
/// 可见（daemon 协议真实入口；driver 持有 daemon 生命周期，kill 收尾
/// ——EOF 优雅关闭语义由 plan269 daemon 族承担，这里不断言；daemon
/// stdio 全 null 防"管道无人读塞死"楔死）。
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
    // driver 拥有 daemon：kill 收尾（daemon 协议自身另有 plan269 族）。
    let _ = daemon.kill();
    let _ = daemon.wait();
}

/// 有界 run（F-816-B-R1-04/R3 残项闭合）：经 owned 执行设施以
/// **ProcessTree** 承载 CLI 子进程——超时/根退出后 `cancel_within` 终止
/// **整棵 owned 树**（持管道后代一并死→管道 EOF→排水线程真停），再在
/// 剩余预算内有界 join；排水写入**共享缓冲**（线程卡住也不丢已收字节）；
/// 回收未完成时**不映射为成功**——分类为超时/未完回收（code=-1）并保留
/// 部分输出。单一总预算覆盖等待/终止/回收。
#[allow(clippy::type_complexity)]
fn run_bounded(
    args: &[&str],
    cwd: &Path,
    env: &[(&str, String)],
    limit: Duration,
) -> (i32, String, String) {
    use std::io::Read;
    use std::sync::{Arc, Mutex};
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
    let out_buf: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let err_buf: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    fn drain_into<R: Read>(mut r: R, buf: Arc<Mutex<String>>) {
        let mut chunk = [0u8; 4096];
        loop {
            match r.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    buf.lock()
                        .unwrap()
                        .push_str(&String::from_utf8_lossy(&chunk[..n]));
                }
            }
        }
    }
    let drains: Vec<_> = [
        proc.take_stdout().map(|r| {
            let b = out_buf.clone();
            std::thread::spawn(move || drain_into(r, b))
        }),
        proc.take_stderr().map(|r| {
            let b = err_buf.clone();
            std::thread::spawn(move || drain_into(r, b))
        }),
    ]
    .into_iter()
    .flatten()
    .collect();
    let deadline = Instant::now() + limit;
    let left = |deadline: Instant| -> Duration {
        deadline
            .saturating_duration_since(Instant::now())
            .max(Duration::from_millis(1))
    };
    // 等待根退出（绝对到期判定：预算尽→终止树→未完回收分类）。
    let code = loop {
        if Instant::now() >= deadline {
            if let Err(e) = proc.cancel_within(left(deadline)) {
                let partial_out = out_buf.lock().unwrap().clone();
                return (
                    -1,
                    partial_out,
                    format!("timeout after {limit:?}; cancel failed: {e}"),
                );
            }
            let partial_out = out_buf.lock().unwrap().clone();
            return (-1, partial_out, format!("timeout after {limit:?}"));
        }
        match proc.wait_deadline(Duration::from_millis(20)) {
            Ok(Some(status)) => break status.code().unwrap_or(-1),
            Ok(None) => {}
            Err(e) => {
                let partial_out = out_buf.lock().unwrap().clone();
                return (-1, partial_out, format!("wait failed: {e}"));
            }
        }
    };
    // 根已退出≠树已空：短宽限内未 EOF 的排水=根外仍有 owned 持有者
    //（持管道后代）——终止整棵 owned 树（EOF 确定性、心跳冻结），并
    // **分类为 timeout 回收**而非沿根退出码报成功（R3 残项 A：持有者
    // 在场即非干净成功；正常短命令排水即刻 EOF，不受影响）。
    let join_bounded = |h: std::thread::JoinHandle<()>, budget: Duration| -> bool {
        let end = Instant::now() + budget;
        while !h.is_finished() && Instant::now() < end {
            std::thread::sleep(Duration::from_millis(5));
        }
        h.is_finished() && h.join().is_ok()
    };
    // 排水宽限：无持有者时 EOF 在根退出瞬间到达。
    let grace_end = Instant::now() + Duration::from_millis(150);
    while Instant::now() < grace_end && drains.iter().any(|h| !h.is_finished()) {
        std::thread::sleep(Duration::from_millis(5));
    }
    let holders_present = drains.iter().any(|h| !h.is_finished());
    if holders_present {
        let tree_reclaim = proc.cancel_within(left(deadline));
        let _ = proc.cancel_within(left(deadline));
        let drains_done: Vec<bool> = drains
            .into_iter()
            .map(|h| join_bounded(h, left(deadline)))
            .collect();
        let out = out_buf.lock().unwrap().clone();
        return (
            -1,
            out,
            format!(
                "timeout reclaim: owned pipe holders survived root exit; tree terminated (tree={:?}, drains={drains_done:?})",
                tree_reclaim.map(|_| "ok").map_err(|e| e.to_string())
            ),
        );
    }
    let drains_done: Vec<bool> = drains
        .into_iter()
        .map(|h| join_bounded(h, left(deadline)))
        .collect();
    let out = out_buf.lock().unwrap().clone();
    let err = err_buf.lock().unwrap().clone();
    if drains_done.iter().any(|d| !d) {
        return (
            -1,
            out,
            format!("reclaim incomplete after {limit:?} (drains={drains_done:?})"),
        );
    }
    (code, out, err)
}

/// V-04：watch 就绪横幅 + 变更触发 SFC 再生。F-816-B-R1-04 残项闭合：
/// ①产物**内容身份**（App.vue 必含变更后文案——保留旧字节/不再再生的
/// 故障宿主必红）；②**单一总预算 20s**：等行/断言/收尾共用同一剩余量
/// ——持续输出宿主最多推迟到总预算即红（原逐行 20s 重置可被无限推迟）；
/// ③RAII 守卫保证任何失败出口（含 unwrap panic）都回收子进程与 reader。
#[test]
fn watch_regenerates_sfc_on_change() {
    let proj = TempProj::new(
        "watch",
        "name: \"wdriver\"
render: \"vue\"
",
        APP,
    );
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
    let pump = std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    if lines_tx.send(line).is_err() {
                        break;
                    }
                }
            }
        }
    });
    let guard = WatchGuard {
        proc: Some(watch),
        pump: Some(pump),
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
        "widget App {
    view {
        col {
            text \"driver v2 marker-9f3a\" {}
        }
    }
}
",
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
    // 内容身份：再生产物必须携带变更后的文案——保留旧字节的故障在此红。
    let sfc_path = proj.path().join("gen/front/vue/src/components/App.vue");
    let sfc = std::fs::read_to_string(&sfc_path)
        .unwrap_or_else(|e| panic!("regenerated SFC must exist at {}: {e}", sfc_path.display()));
    if !sfc.contains("driver v2 marker-9f3a") {
        panic!(
            "regenerated SFC must carry the changed text, got:
{sfc}"
        );
    }
    guard.release();
}

/// V-03/V-04：`auto gen` 经真实工具进程（I-TOOL generate_ui 五操作链）。
/// AUTO_UI_TOOL_BIN 未设时尝试主 workspace target/debug 产物；缺失=
/// 诚实红灯（不静默跳过）。
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

fn wait_bounded(child: &mut Child, limit: Duration) -> bool {
    let deadline = Instant::now() + limit;
    loop {
        if let Some(_) = child.try_wait().expect("try_wait") {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

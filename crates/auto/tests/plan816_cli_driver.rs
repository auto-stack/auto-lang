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

/// 有界 run（F-816-B-R1-04）：stdout/stderr 由**并发排水线程**持续收
/// 集（先等退出再读管道会被写满管道的子进程死锁）；超时 kill 并按超时
/// 失败报告——所有失败出口都保证子进程收尾。
fn run_bounded(
    args: &[&str],
    cwd: &Path,
    env: &[(&str, String)],
    limit: Duration,
) -> (i32, String, String) {
    use std::io::Read;
    let mut cmd = Command::new(AUTO);
    cmd.args(args).current_dir(cwd).stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    for (k, v) in env {
        cmd.env(k, v);
    }
    let mut child = cmd
        .spawn()
        .unwrap_or_else(|e| panic!("spawn auto {:?}: {e}", args));
    fn drain(mut r: std::process::ChildStdout) -> String {
        let mut b = String::new();
        let _ = r.read_to_string(&mut b);
        b
    }
    fn drain_err(mut r: std::process::ChildStderr) -> String {
        let mut b = String::new();
        let _ = r.read_to_string(&mut b);
        b
    }
    let out_handle = child
        .stdout
        .take()
        .map(|r| std::thread::spawn(move || drain(r)));
    let err_handle = child
        .stderr
        .take()
        .map(|r| std::thread::spawn(move || drain_err(r)));
    let deadline = Instant::now() + limit;
    let code = loop {
        match child.try_wait().expect("try_wait") {
            Some(status) => break status.code().unwrap_or(-1),
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return (
                    -1,
                    out_handle.and_then(|h| h.join().ok()).unwrap_or_default(),
                    format!("timeout after {limit:?}"),
                );
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    };
    // 进程已退：排水线程随 EOF 结束，有界 join。
    let out = out_handle
        .and_then(|h| h.join().ok())
        .unwrap_or_default();
    let err = err_handle
        .and_then(|h| h.join().ok())
        .unwrap_or_default();
    (code, out, err)
}

/// V-04：watch 就绪横幅 + 变更触发 SFC 再生。F-816-B-R1-04 的区分性
/// oracle：①不只看进度文本——**产物内容身份**（App.vue 必须含变更后的
/// 文案，"保留旧字节/不再再生"的故障宿主必须红）；②阻塞读经行通道
/// `recv_timeout` 包住（deadline 真正生效，read_line 不再裸阻塞）；
/// ③所有失败出口先 kill 收尾再 panic（不残留子进程）。
#[test]
fn watch_regenerates_sfc_on_change() {
    let proj = TempProj::new("watch", "name: \"wdriver\"
render: \"vue\"
", APP);
    let mut watch = Command::new(AUTO)
        .args(["watch"])
        .current_dir(proj.path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn watch");
    // 行通道：reader 线程搬运，主循环 recv_timeout=真截止。
    let (lines_tx, lines_rx) = std::sync::mpsc::channel::<String>();
    let stdout = watch.stdout.take().expect("piped stdout");
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
    // 收尾 guard：任何失败出口先 kill 子进程（pump 经 Option 单次取用，
    // 宏展开多处调用不重复 move）。
    let mut pump = Some(pump);
    let mut watch = Some(watch);
    macro_rules! finish_watch {
        () => {{
            if let Some(mut w) = watch.take() {
                let _ = w.kill();
                let _ = w.wait();
            }
            if let Some(p) = pump.take() {
                let _ = p.join();
            }
        }};
    }
    let recv_bounded = |limit: Duration| -> Option<String> {
        lines_rx.recv_timeout(limit).ok()
    };

    let mut ready = false;
    let deadline = Duration::from_secs(20);
    while let Some(line) = recv_bounded(deadline) {
        if line.contains("Ready. Edit an .at file") {
            ready = true;
            break;
        }
    }
    if !ready {
        finish_watch!();
        panic!("watch ready banner expected within 20s");
    }
    // 变更：文案 v1 → v2（内容身份的载体）。
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
    .unwrap();
    let mut regen = false;
    let mut last = String::new();
    while let Some(line) = recv_bounded(Duration::from_secs(20)) {
        last = line;
        if last.contains("SFC") && last.contains("app.at") {
            regen = true;
            break;
        }
    }
    if !regen {
        finish_watch!();
        panic!("change must regenerate the SFC: last={last}");
    }
    // 内容身份：再生产物必须携带变更后的文案——保留旧字节的故障在此红。
    let sfc_path = proj.path().join("gen/front/vue/src/components/App.vue");
    let sfc = std::fs::read_to_string(&sfc_path)
        .unwrap_or_else(|e| {
            finish_watch!();
            panic!("regenerated SFC must exist at {}: {e}", sfc_path.display())
        });
    if !sfc.contains("driver v2 marker-9f3a") {
        finish_watch!();
        panic!("regenerated SFC must carry the changed text, got:
{sfc}");
    }
    finish_watch!();
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

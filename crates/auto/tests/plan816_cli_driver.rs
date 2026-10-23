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

/// 有界 run：超时 kill 并按超时失败报告（防 req 对楔死目标悬挂）。
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
    let deadline = Instant::now() + limit;
    loop {
        match child.try_wait().expect("try_wait") {
            Some(status) => {
                let mut out = String::new();
                let mut err = String::new();
                if let Some(mut r) = child.stdout.take() {
                    let _ = r.read_to_string(&mut out);
                }
                if let Some(mut r) = child.stderr.take() {
                    let _ = r.read_to_string(&mut err);
                }
                return (status.code().unwrap_or(-1), out, err);
            }
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return (-1, String::new(), format!("timeout after {limit:?}"));
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    }
}

/// V-04：watch 就绪横幅 + 变更触发 SFC 再生（Vite HMR 的前置产物）+
/// 进程终止退出（V-02 watch 变更切片的真 CLI 面）。
#[test]
fn watch_regenerates_sfc_on_change() {
    let proj = TempProj::new("watch", "name: \"wdriver\"\nrender: \"vue\"\n", APP);
    let mut watch = Command::new(AUTO)
        .args(["watch"])
        .current_dir(proj.path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn watch");
    let mut stdout = BufReader::new(watch.stdout.take().expect("piped stdout"));
    let mut line = String::new();
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut ready = false;
    while Instant::now() < deadline {
        line.clear();
        if stdout.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        if line.contains("Ready. Edit an .at file") {
            ready = true;
            break;
        }
    }
    assert!(ready, "watch ready banner expected");
    std::fs::write(
        proj.path().join("src/front/app.at"),
        "widget App {\n    view {\n        col {\n            text \"driver v2\" {}\n        }\n    }\n}\n",
    )
    .unwrap();
    let mut regen = false;
    let deadline = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline {
        line.clear();
        if stdout.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        if line.contains("SFC") && line.contains("app.at") {
            regen = true;
            break;
        }
    }
    assert!(regen, "change must regenerate the SFC: last={line}");
    assert!(
        proj.path()
            .join("gen/front/vue/src/components/App.vue")
            .exists(),
        "regenerated SFC artifact must exist"
    );
    let _ = watch.kill();
    let _ = watch.wait();
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

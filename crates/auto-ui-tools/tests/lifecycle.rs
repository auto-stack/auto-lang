//! PLAN-815 T-07: upper-layer lifecycle integration tests — exercise the
//! full generate → validate → run → build sequence over the real tool
//! process, proving the operations compose correctly (V-01..04 oracle).
//!
//! PLAN-815 T-06..08 R1（F-815-T07-R1-01/02）重写：测试辅助器并发排水
//! stderr（管道满不再堵死验收）+ 全部等待有界（超时可靠清理 owned 子
//! 树并带日志尾巴诊断）；build oracle 分正负——成功=真实可消费 dist
//! 资产+内容，安装确定失败=具名 failed 且**不得**通过成功面断言。

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use auto_project_model::descriptor::SourceInfo;
use auto_project_model::{parse_document, GenerationRequest};
use auto_tool_protocol::message::{
    ClientBody, ClientEnvelope, ServerBody, ServerEnvelope, WireRequest,
};
use auto_tool_protocol::{Budgets, RunOutcome};
use serde_json::Value as Json;

const BIN: &str = env!("CARGO_BIN_EXE_auto-ui-tool");
/// Bounded wait for any control-stream answer (the tool answers framed
/// requests in well under a second; 30s covers cold generation).
const ANSWER_BOUND: Duration = Duration::from_secs(30);
/// Bounded wait for the child's exit after EOF-close (T-06..08 R2:
/// close/清理路径同样有界).
const CLOSE_BOUND: Duration = Duration::from_secs(10);

struct Tool {
    child: Child,
    /// stdout reader thread feed (framed control stream).
    stdout_rx: mpsc::Receiver<Vec<u8>>,
    /// stderr drained concurrently by a reader thread (F-815-T07-R1-02:
    /// a piped-but-unread stderr fills the OS pipe and blocks the whole
    /// chain — 131072 bytes of ordinary logs wedged the old helper).
    stderr_log: Arc<Mutex<Vec<u8>>>,
    buffer: Vec<u8>,
    pos: usize,
}

impl Tool {
    fn spawn() -> Self {
        Self::spawn_with_env(None)
    }

    /// Spawn with extra env (e.g. a stubbed PATH for deterministic
    /// package-manager control).
    fn spawn_with_env(extra_env: Option<Vec<(String, String)>>) -> Self {
        let mut child = {
            let mut command = Command::new(BIN);
            command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            if let Some(env) = &extra_env {
                for (key, value) in env {
                    command.env(key, value);
                }
            }
            command.spawn().expect("spawn")
        };
        let stdout = child.stdout.take().expect("stdout");
        let stderr = child.stderr.take().expect("stderr");
        let (tx, stdout_rx) = mpsc::channel::<Vec<u8>>();
        std::thread::Builder::new()
            .name("lifecycle-stdout".into())
            .spawn(move || {
                let mut stdout = stdout;
                let mut chunk = [0u8; 8192];
                loop {
                    match stdout.read(&mut chunk) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            if tx.send(chunk[..n].to_vec()).is_err() {
                                break;
                            }
                        }
                    }
                }
            })
            .expect("spawn stdout reader");
        let stderr_log = Arc::new(Mutex::new(Vec::new()));
        let stderr_sink = Arc::clone(&stderr_log);
        std::thread::Builder::new()
            .name("lifecycle-stderr".into())
            .spawn(move || {
                let mut stderr = stderr;
                let mut chunk = [0u8; 8192];
                loop {
                    match stderr.read(&mut chunk) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => stderr_sink.lock().unwrap().extend_from_slice(&chunk[..n]),
                    }
                }
            })
            .expect("spawn stderr drain");
        Self {
            child,
            stdout_rx,
            stderr_log,
            buffer: Vec::new(),
            pos: 0,
        }
    }

    fn stderr_so_far(&self) -> String {
        String::from_utf8_lossy(&self.stderr_log.lock().unwrap()).into_owned()
    }

    fn send(&mut self, message: &Json) {
        let stdin = self.child.stdin.as_mut().expect("stdin");
        auto_tool_protocol::framing::write_frame(
            stdin,
            message,
            Budgets::contract().max_frame_bytes,
        )
        .expect("write frame");
    }

    /// One framed response, bounded: on the bound the owned child tree is
    /// killed and the panic carries the stderr tail (F-815-T07-R1-02 —
    /// no unbounded reads, no wedged pipes).
    fn next_response_bounded(&mut self) -> Json {
        let deadline = Instant::now() + ANSWER_BOUND;
        loop {
            let buffer = self.buffer[self.pos..].to_vec();
            match auto_tool_protocol::framing::parse_frame(
                &buffer,
                Budgets::contract().max_frame_bytes,
            ) {
                auto_tool_protocol::framing::FrameParse::Complete { message, consumed } => {
                    self.pos += consumed;
                    return message;
                }
                auto_tool_protocol::framing::FrameParse::Incomplete { .. } => {}
                other => panic!("bad frame: {other:?}"),
            }
            match self
                .stdout_rx
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            {
                Ok(chunk) => self.buffer.extend_from_slice(&chunk),
                Err(_) => {
                    let tail = self.stderr_tail();
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    panic!("no framed response within {ANSWER_BOUND:?}; stderr tail: {tail}");
                }
            }
        }
    }

    fn stderr_tail(&self) -> String {
        let log = self.stderr_so_far();
        let lines: Vec<&str> = log.lines().collect();
        let n = lines.len();
        lines[n.saturating_sub(12)..].join(" | ")
    }

    /// Close stdin and wait for the exit, BOUNDED (PLAN-815 T-06..08 R2 /
    /// F-815-T07-R1-02：close 路径不再无界 Child::wait——EOF 后不退出的
    /// 被测对象由辅助器自身收口并携带诊断，不依赖外部强杀）。
    fn close_and_wait(mut self) -> i32 {
        drop(self.child.stdin.take());
        let deadline = Instant::now() + CLOSE_BOUND;
        loop {
            match self.child.try_wait().expect("try_wait") {
                Some(status) => {
                    // Give the drain threads a moment to finish.
                    std::thread::sleep(Duration::from_millis(50));
                    return status.code().unwrap_or(-1);
                }
                None => {
                    if Instant::now() > deadline {
                        let tail = self.stderr_tail();
                        let _ = self.child.kill();
                        let _ = self.child.wait();
                        panic!(
                            "tool did not exit within {CLOSE_BOUND:?} after EOF; stderr tail: {tail}"
                        );
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
        }
    }
}

impl Drop for Tool {
    fn drop(&mut self) {
        // Reliable cleanup for the owned child on ANY exit path (assert
        // failure included), with a bounded post-kill reap — the cleanup
        // path itself never blocks forever either.
        let _ = self.child.kill();
        let deadline = Instant::now() + CLOSE_BOUND;
        while self.child.try_wait().map(|s| s.is_none()).unwrap_or(false) {
            if Instant::now() > deadline {
                eprintln!("lifecycle Tool Drop: child did not reap within {CLOSE_BOUND:?}");
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

fn envelope(id: u64, body: ClientBody) -> Json {
    serde_json::to_value(ClientEnvelope {
        protocol_version: auto_tool_protocol::PROTOCOL_VERSION,
        id,
        body,
    })
    .expect("serialize envelope")
}

fn parse_response(json: &Json) -> ServerEnvelope {
    serde_json::from_value(json.clone()).expect("typed response")
}

fn write_vue_project(root: &std::path::Path, name: &str, pac: &str) {
    std::fs::create_dir_all(root.join("src").join("front")).unwrap();
    std::fs::write(root.join("pac.at"), pac).unwrap();
    std::fs::write(
        root.join("src").join("front").join("app.at"),
        "widget App {\n    model { var n int = 0 }\n    view { col { text \"${.n}\" } }\n}\n",
    )
    .unwrap();
    let _ = name;
}

/// V-01/V-02 lifecycle: generate a real project → validate the generated
/// project → run it and observe the honest backend failure (the fixture
/// declares no backend contract). This proves the operations compose:
/// generate's artifacts are visible to validate (real paths), and run
/// exercises the real generation chain before the honest failure.
#[test]
#[ignore = "heavy: real generation chain; serial evidence run via --ignored"]
fn lifecycle_generate_validate_run() {
    let dir = tempfile_dir();
    let root = dir.path();
    write_vue_project(
        root,
        "lifecycle",
        "name: \"lifecycle\"\nrender: \"vue\"\nwindow: \"480x360\"\nfront: \"./src/front\"\n",
    );

    let request = GenerationRequest::from_descriptor(
        &parse_document(
            "name: \"lifecycle\"\nrender: \"vue\"\nwindow: \"480x360\"\nfront: \"./src/front\"\n",
            SourceInfo::from_path(root.join("pac.at")),
        )
        .expect("parse"),
    );

    let mut tool = Tool::spawn();

    // 1) Generate: real artifacts land on disk.
    tool.send(&envelope(
        1,
        ClientBody::GenerateUi {
            request: WireRequest(request.clone()),
            output: None,
            project: None,
            delivery_root: None,
            strict: None,
        },
    ));
    let (gen_final, _) = wait_for(&mut tool, |e| {
        matches!(e.body, ServerBody::GenerateOk { .. })
            || matches!(e.body, ServerBody::Error { .. })
    });
    match gen_final.body {
        ServerBody::GenerateOk { report } => {
            assert!(report.files_written > 5, "real artifacts: {report:?}");
            assert!(
                root.join("gen").join("front").join("vue").exists(),
                "generation root on disk"
            );
        }
        other => panic!("expected GenerateOk, got {other:?}"),
    }

    // 2) Validate the SAME project: paths now exist → ok report.
    tool.send(&envelope(
        2,
        ClientBody::ValidateProject {
            request: WireRequest(request.clone()),
        },
    ));
    let resp = parse_response(&tool.next_response_bounded());
    match resp.body {
        ServerBody::ValidateOk { report } => {
            assert!(
                report.ok,
                "generated project passes validation: {:?}",
                report.findings
            );
        }
        other => panic!("expected ValidateOk, got {other:?}"),
    }

    // 3) Run: exercises the real generation+install chain, then the honest
    // backend failure (no api.at declared → start_api_server refuses).
    tool.send(&envelope(
        3,
        ClientBody::RunUi {
            request: WireRequest(request),
            args: Vec::new(),
        },
    ));
    let (run_final, _) = wait_for(&mut tool, |e| {
        matches!(e.body, ServerBody::RunOk { .. }) || matches!(e.body, ServerBody::Error { .. })
    });
    match run_final.body {
        ServerBody::RunOk { report } => {
            assert_eq!(report.outcome, RunOutcome::Failed);
            assert!(
                report.error.as_deref().unwrap_or("").contains("backend"),
                "honest backend failure: {:?}",
                report.error
            );
        }
        other => panic!("expected RunOk, got {other:?}"),
    }

    // 4) The tool keeps serving A ops after the full lifecycle.
    tool.send(&envelope(4, ClientBody::Describe));
    assert!(matches!(
        parse_response(&tool.next_response_bounded()).body,
        ServerBody::DescribeOk { .. }
    ));

    let code = tool.close_and_wait();
    assert_eq!(code, 0);
}

/// V-04 build oracle — SUCCESS face (F-815-T07-R1-01 重写)：build_ui on a
/// freshly installed vue project runs the real `pnpm run build` and the
/// oracle is the CONSUMABLE dist output itself: `dist/index.html` exists,
/// references real entry assets, and the report is completed/error=None.
/// Anything else (named rejection, failed, missing dist) FAILS this test.
#[test]
#[ignore = "heavy+env: real pnpm install + build; serial evidence run via --ignored"]
fn lifecycle_build_produces_dist() {
    let dir = tempfile_dir();
    let root = dir.path();
    write_vue_project(
        root,
        "buildtest",
        "name: \"buildtest\"\nrender: \"vue\"\nwindow: \"480x360\"\nfront: \"./src/front\"\n",
    );

    let request = GenerationRequest::from_descriptor(
        &parse_document(
            "name: \"buildtest\"\nrender: \"vue\"\nwindow: \"480x360\"\nfront: \"./src/front\"\n",
            SourceInfo::from_path(root.join("pac.at")),
        )
        .expect("parse"),
    );

    let mut tool = Tool::spawn();

    // Run first (real install chain), then the honest backend failure —
    // node_modules now exists for the build.
    tool.send(&envelope(
        10,
        ClientBody::RunUi {
            request: WireRequest(request.clone()),
            args: Vec::new(),
        },
    ));
    let (run_final, _) = wait_for(&mut tool, |e| matches!(&e.body, ServerBody::RunOk { .. }));
    match &run_final.body {
        ServerBody::RunOk { report } => assert_eq!(report.outcome, RunOutcome::Failed),
        other => panic!("expected RunOk, got {other:?}"),
    }

    // Build: the REAL production build must complete and produce the
    // consumable dist (F-815-T07-R1-01: a failed/errored build must NOT
    // pass this success oracle).
    tool.send(&envelope(
        11,
        ClientBody::BuildUi {
            request: WireRequest(request),
            strict: None,
        },
    ));
    let (build_final, _) = wait_for(&mut tool, |e| {
        matches!(e.body, ServerBody::BuildOk { .. }) || matches!(e.body, ServerBody::Error { .. })
    });
    match build_final.body {
        ServerBody::BuildOk { report } => {
            assert_eq!(
                report.outcome,
                RunOutcome::Completed,
                "success oracle requires a completed build: {report:?} stderr: {}",
                tool.stderr_tail()
            );
            assert_eq!(report.error, None, "completed build carries no error");
            assert_eq!(report.profile, "vue");
        }
        ServerBody::Error { error } => {
            panic!(
                "build_ui was rejected by name — success oracle fails: {error:?} stderr: {}",
                tool.stderr_tail()
            );
        }
        other => panic!("expected BuildOk, got {other:?}"),
    }

    // The oracle is the real artifact: dist/index.html exists and
    // references entry assets on disk.
    let dist = root.join("gen").join("front").join("vue").join("dist");
    let index = dist.join("index.html");
    let html = std::fs::read_to_string(&index)
        .unwrap_or_else(|e| panic!("dist/index.html missing ({e}): {}", tool.stderr_tail()));
    assert!(
        html.contains("<script") || html.contains("<div id"),
        "index.html is a real entry document: {html}"
    );
    // Content oracle (PLAN-815 T-06..08 R2 / F-815-T07-R1-01 强化):
    // success = real bundles on disk AND the entry document REALLY
    // references them — EVERY assets/ reference must resolve to an
    // existing file and at least one must exist. A broken reference or a
    // reference-free entry FAILS the success oracle (no `||` fallback).
    let assets_dir = dist.join("assets");
    let mut bundles = 0u32;
    if let Ok(entries) = std::fs::read_dir(&assets_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.ends_with(".js") && entry.path().is_file() {
                bundles += 1;
            }
        }
    }
    assert!(
        bundles > 0,
        "dist/assets carries real JS bundles: {}",
        tool.stderr_tail()
    );
    let mut references = 0u32;
    let mut js_references = 0u32;
    for token in html.split(['"', '\'', '(', ')', ' ']) {
        let Some(pos) = token.find("assets/") else {
            continue;
        };
        let name = token[pos + "assets/".len()..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '/'))
            .collect::<String>();
        if name.is_empty() {
            // A bare "assets/" dir reference carries no file claim.
            continue;
        }
        assert!(
            assets_dir.join(&name).is_file(),
            "entry references a MISSING dist asset: {name} (broken build output): {html}"
        );
        references += 1;
        if name.ends_with(".js") {
            js_references += 1;
        }
    }
    assert!(
        references > 0,
        "entry document references at least one real dist bundle: {html}"
    );
    // The entry must actually LOAD the application JS — a CSS-only entry
    // (or any no-JS-load shape) is not a consumable bundle success
    // (PLAN-815 T-05 R8 / F-815-T07-R1-01: "some .js exists in the dir" +
    // "some asset referenced" were two unlinked checks).
    assert!(
        js_references > 0,
        "entry document loads the application JS (a .js asset reference): {html}"
    );

    let code = tool.close_and_wait();
    assert_eq!(code, 0);
}

/// V-04 build oracle — FAILURE face (F-815-T07-R1-01)：安装确定失败
/// （私有 stub 包管理器，请求 `pkg:` 固定，零网络）时 build 必须是具名
/// `failed`、不产生 dist，且**不得**通过上面的成功面断言。同时保持
/// stderr 有日志时的收口（F-815-T07-R1-02 的辅助器排水）。
#[test]
#[ignore = "heavy: real generation chain + stub PM; serial evidence run via --ignored"]
fn lifecycle_build_install_failure_is_named_failed_no_dist() {
    let dir = tempfile_dir();
    let root = dir.path();
    write_vue_project(
        root,
        "buildfail",
        "name: \"buildfail\"\nrender: \"vue\"\nwindow: \"480x360\"\nfront: \"./src/front\"\npkg: \"pnpm\"\n",
    );
    // The request carries `pkg: "pnpm"` (raw prop) — resolution is
    // request-sourced, so the stub PM below is selected deterministically.
    let request = GenerationRequest::from_descriptor(
        &parse_document(
            "name: \"buildfail\"\nrender: \"vue\"\nwindow: \"480x360\"\nfront: \"./src/front\"\npkg: \"pnpm\"\n",
            SourceInfo::from_path(root.join("pac.at")),
        )
        .expect("parse"),
    );

    // Stub pnpm on the tool's PATH: records the call, exits 7 immediately.
    let stub_dir = tempfile_dir();
    let marker = stub_dir.path().join("pnpm-called");
    #[cfg(windows)]
    std::fs::write(
        stub_dir.path().join("pnpm.cmd"),
        format!(
            "@echo called>>\"{}\"\r\n@exit /b 7\r\n",
            marker.to_string_lossy().replace('\\', "/")
        ),
    )
    .unwrap();
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;
        let stub = stub_dir.path().join("pnpm");
        std::fs::write(
            &stub,
            format!("#!/bin/sh\necho called > '{}'\nexit 7\n", marker.display()),
        )
        .unwrap();
        let mut perms = std::fs::metadata(&stub).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&stub, perms).unwrap();
    }
    let system = if cfg!(windows) {
        std::env::var_os("SystemRoot")
            .map(|r| std::path::PathBuf::from(r).join("System32"))
            .unwrap_or_default()
    } else {
        std::path::PathBuf::from("/usr/bin")
    };
    let mut path = std::ffi::OsString::from(stub_dir.path());
    path.push(if cfg!(windows) { ";" } else { ":" });
    path.push(system.as_os_str());

    let mut tool = Tool::spawn_with_env(Some(vec![(
        "PATH".to_string(),
        path.to_string_lossy().into_owned(),
    )]));

    tool.send(&envelope(
        20,
        ClientBody::BuildUi {
            request: WireRequest(request),
            strict: None,
        },
    ));
    let (build_final, _) = wait_for(&mut tool, |e| {
        matches!(e.body, ServerBody::BuildOk { .. }) || matches!(e.body, ServerBody::Error { .. })
    });
    match build_final.body {
        ServerBody::BuildOk { report } => {
            assert_eq!(
                report.outcome,
                RunOutcome::Failed,
                "install exit7 must be a named failure: {report:?}"
            );
            assert!(
                report.error.as_deref().unwrap_or("").contains("install"),
                "named install failure: {:?}",
                report.error
            );
        }
        other => panic!("expected BuildOk(failed), got {other:?}"),
    }
    // The failure oracle: no dist produced, and the stub PM WAS invoked
    // (the deterministic negative really reached the install step).
    assert!(
        !root
            .join("gen")
            .join("front")
            .join("vue")
            .join("dist")
            .exists(),
        "failed install must not produce dist"
    );
    assert!(marker.exists(), "stub pnpm was actually invoked");
    // Noisy-stderr robustness (F-815-T07-R1-02): the drained stderr never
    // wedged the bounded waits above.
    let code = tool.close_and_wait();
    assert_eq!(code, 0);
}

/// T-07 子批2 V-04：Unicode+空格工程（目录 "示例 app"、包名 "示例应用"、
/// zh 标题）走真实 install+build 链——成功面 oracle 是可消费 dist（真实
/// JS bundle 且入口真实引用，与 lifecycle_build_produces_dist 同标准），
/// 且 dist 入口携带 zh 标题。Unicode/空格路径是一等工程形态。
#[test]
#[ignore = "heavy+env: real pnpm install + build under unicode/space paths; serial evidence run via --ignored"]
fn lifecycle_unicode_space_project_builds_dist() {
    let dir = tempfile_dir();
    // Unicode + space project directory as the real project root.
    let root = dir.path().join("示例 app");
    write_vue_project(
        &root,
        "示例应用",
        "name: \"示例应用\"\ntitle_zh: \"中文标题示例\"\nrender: \"vue\"\nwindow: \"480x360\"\nfront: \"./src/front\"\n",
    );

    let request = GenerationRequest::from_descriptor(
        &parse_document(
            "name: \"示例应用\"\ntitle_zh: \"中文标题示例\"\nrender: \"vue\"\nwindow: \"480x360\"\nfront: \"./src/front\"\n",
            SourceInfo::from_path(root.join("pac.at")),
        )
        .expect("parse"),
    );

    let mut tool = Tool::spawn();

    // Generate first (real chain), then the REAL install+build in one op.
    tool.send(&envelope(
        30,
        ClientBody::GenerateUi {
            request: WireRequest(request.clone()),
            output: None,
            project: None,
            delivery_root: None,
            strict: None,
        },
    ));
    let (gen_final, _) = wait_for(&mut tool, |e| {
        matches!(e.body, ServerBody::GenerateOk { .. })
            || matches!(e.body, ServerBody::Error { .. })
    });
    match gen_final.body {
        ServerBody::GenerateOk { report } => {
            assert!(report.files_written > 5, "real artifacts: {report:?}");
        }
        other => panic!("expected GenerateOk, got {other:?}"),
    }

    tool.send(&envelope(
        31,
        ClientBody::BuildUi {
            request: WireRequest(request),
            strict: None,
        },
    ));
    let (build_final, _) = wait_for(&mut tool, |e| {
        matches!(e.body, ServerBody::BuildOk { .. }) || matches!(e.body, ServerBody::Error { .. })
    });
    match build_final.body {
        ServerBody::BuildOk { report } => {
            assert_eq!(
                report.outcome,
                RunOutcome::Completed,
                "unicode/space project must complete the real build: {report:?} stderr: {}",
                tool.stderr_tail()
            );
            assert_eq!(report.error, None);
            assert_eq!(report.profile, "vue");
        }
        ServerBody::Error { error } => {
            panic!(
                "build_ui rejected by name for the unicode/space project: {error:?} stderr: {}",
                tool.stderr_tail()
            );
        }
        other => panic!("expected BuildOk, got {other:?}"),
    }

    // Consumable-dist oracle (same standard as the ASCII lifecycle build):
    // dist/index.html exists, carries the zh title, and REALLY loads the
    // application JS — every referenced assets/ file exists on disk.
    let dist = root.join("gen").join("front").join("vue").join("dist");
    let html = std::fs::read_to_string(dist.join("index.html"))
        .unwrap_or_else(|e| panic!("dist/index.html missing ({e}): {}", tool.stderr_tail()));
    assert!(
        html.contains("<title>中文标题示例</title>"),
        "zh title travels into the built dist: {html}"
    );
    let assets_dir = dist.join("assets");
    let mut references = 0u32;
    let mut js_references = 0u32;
    for token in html.split(['"', '\'', '(', ')', ' ']) {
        let Some(pos) = token.find("assets/") else {
            continue;
        };
        let name = token[pos + "assets/".len()..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '/'))
            .collect::<String>();
        if name.is_empty() {
            continue;
        }
        assert!(
            assets_dir.join(&name).is_file(),
            "entry references a MISSING dist asset: {name}: {html}"
        );
        references += 1;
        if name.ends_with(".js") {
            js_references += 1;
        }
    }
    assert!(references > 0, "entry references real bundles: {html}");
    assert!(js_references > 0, "entry loads the application JS: {html}");

    let code = tool.close_and_wait();
    assert_eq!(code, 0);
}

fn wait_for(
    tool: &mut Tool,
    mut predicate: impl FnMut(&ServerEnvelope) -> bool,
) -> (ServerEnvelope, Vec<ServerEnvelope>) {
    let mut seen = Vec::new();
    loop {
        let envelope = parse_response(&tool.next_response_bounded());
        if predicate(&envelope) {
            seen.push(envelope.clone());
            return (envelope, seen);
        }
        seen.push(envelope);
    }
}

fn tempfile_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

//! PLAN-816 T-12 V-05：M816-A I-TOOL 客户端的独立消费方探针。
//!
//! 编译期：只依赖 `auto-tool-client`（+ 构造请求的 `auto-project-model`）
//! ——客户端闭包内不得出现 auto-ui-tools（工具实现只经真实进程消费）、
//! auto-man/auto-lang/auto-vm/auto-gen/auto-media-core（"Core 无 UI 安装
//! 可消费"；由 `verify-deps.sh` 的 cargo tree 断言钉住）。
//!
//! 运行期：经客户端 API 消费**已发布 M815-A** 的真实 `auto-ui-tool`
//! 进程——describe/validate 窄能力正控、版本/能力拒绝零副作用、cancel
//! 诚实语义、EOF 优雅关闭、控制流/日志流隔离；寿命反例（超时/孙进程/
//! reader 真停/流污染）用自带截止的具名测试子进程 te-child 补充（真实
//! 窄 tool 不能代替寿命反例）。不切换任何默认 CLI。

#[cfg(test)]
use std::time::{Duration, Instant};

#[cfg(test)]
use auto_project_model::descriptor::SourceInfo;
#[cfg(test)]
use auto_project_model::{parse_document, GenerationRequest};
#[cfg(test)]
use auto_tool_client::discovery::{discover, DiscoveryError, DiscoverySpec};
#[cfg(test)]
use auto_tool_client::{BrokenReason, ClientError, ExitKind, Reply, ToolClient, ToolClientSpec};
#[cfg(test)]
use auto_tool_execution::TreeScope;
#[cfg(test)]
use auto_tool_protocol::message::{ClientBody, WireRequest};
#[cfg(test)]
use auto_tool_protocol::{ErrorKind, FrameError, PROTOCOL_VERSION};

/// 真实工具二进制：env 覆盖 → 主 workspace 默认产物路径；缺失即失败
/// （诚实红灯，不静默跳过——先 `cargo build -p auto-ui-tools`）。
#[cfg(test)]
fn tool_bin() -> std::path::PathBuf {
    binary("AUTO_UI_TOOL_BIN", "auto-ui-tool")
}

/// 具名测试子进程：env 覆盖 → 主 workspace 默认产物路径。
#[cfg(test)]
fn te_child() -> std::path::PathBuf {
    binary("TE_CHILD_BIN", "te-child")
}

#[cfg(test)]
fn binary(env_key: &str, name: &str) -> std::path::PathBuf {
    if let Some(path) = std::env::var_os(env_key) {
        return path.into();
    }
    let exe = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../target/debug")
        .join(exe);
    assert!(
        path.exists(),
        "{name} binary not found at {} — build it first: \
         cargo build -p auto-ui-tools -p auto-tool-execution (or set {env_key})",
        path.display()
    );
    path
}

#[cfg(test)]
fn real_client() -> ToolClient {
    ToolClient::spawn(ToolClientSpec::new(tool_bin())).expect("spawn real auto-ui-tool")
}

/// 在给定（调用方持有、请求期间存活）目录里构造样例请求。
#[cfg(test)]
fn sample_request(
    base: &std::path::Path,
    output_root: Option<&std::path::Path>,
) -> GenerationRequest {
    let pac = base.join("pac.at");
    std::fs::write(&pac, "name: \"外部 demo\"\nrender: \"vue\"\n").unwrap();
    let mut request = GenerationRequest::from_descriptor(
        &parse_document(
            "name: \"外部 demo\"\nrender: \"vue\"\n",
            SourceInfo::from_path(&pac),
        )
        .expect("parse"),
    );
    request.output_root = output_root.map(|p| p.to_path_buf());
    request
}

/// V-05 正控：describe 经真实进程；EOF 优雅关闭码 0；日志只在 stderr
/// 且不含任何控制帧内容（流隔离负控）。
#[test]
fn consumer_describes_real_tool_via_client() {
    let mut client = real_client();
    let descriptor = client.describe().expect("describe over real process");
    assert_eq!(descriptor.tool_name, "auto-ui-tool");
    assert_eq!(descriptor.protocol_version, PROTOCOL_VERSION);
    assert_eq!(
        descriptor
            .capabilities
            .iter()
            .map(|c| c.operation.as_str())
            .collect::<Vec<_>>(),
        // M815-B 发布 I-TOOL v1 五操作全集（A 期为 describe/validate_project
        // 窄集）；816 消费交接=五操作，此处钉已发布全集。
        vec![
            "describe",
            "validate_project",
            "generate_ui",
            "build_ui",
            "run_ui"
        ]
    );
    // stderr 与 stdout 是两条异步流:describe 返回不保证 stderr 快照已
    // 含日志行——以 shutdown 后（drain 完成）的日志断言为准。
    let shutdown = client.shutdown().expect("clean EOF shutdown");
    assert_eq!(shutdown.kind, ExitKind::Success);
    assert!(
        shutdown.logs.contains("accepted request"),
        "stderr: {:?}",
        shutdown.logs
    );
    // 控制帧（DescribeOk 载荷）绝不出现在日志流。
    assert!(!shutdown.logs.contains("DescribeOk"));
    assert!(!shutdown.logs.contains("capabilities"));
}

/// V-05 正控：validate_project 经真实进程，损耗敏感回显。
#[test]
fn consumer_validates_real_request_via_client() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut client = real_client();
    let report = client
        .validate_project(sample_request(dir.path(), None))
        .expect("client round trip")
        .expect("host accepts a well-formed request");
    assert!(report.ok, "findings: {:?}", report.findings);
    assert_eq!(report.summary.name.as_deref(), Some("外部 demo"));
    assert!(report
        .summary
        .not_checked
        .contains(&"generation".to_string()));
    client.shutdown().expect("shutdown");
}

/// M816-B 五操作正控：generate_ui 经真实进程——合成 vue 工程在声明
/// 输出根下真实生成（文件数>0、frontends 含 vue），事件面无虚假。
#[test]
fn consumer_generates_real_project_via_client() {
    let out_root = tempfile::tempdir().expect("tempdir");
    let project = tempfile::tempdir().expect("tempdir");
    // 真实工程布局：工具 vue 生成消费 src/front/app.at 入口（裸 pac
    // 无 front 目录=具名拒绝——本测试要的是生成正控）。
    std::fs::create_dir_all(project.path().join("src").join("front")).expect("front dir");
    std::fs::write(
        project.path().join("src").join("front").join("app.at"),
        "widget App {
    view {
        col {
            text \"gen smoke\" {}
        }
    }
}
",
    )
    .expect("app.at");
    let mut client = real_client();
    let report = client
        .generate_ui(sample_request(project.path(), Some(out_root.path())))
        .expect("client round trip")
        .expect("host generates a well-formed request");
    assert!(report.files_written > 0, "report: {report:?}");
    assert!(report.frontends.contains(&"vue".to_string()), "{report:?}");
    // 815 字段矩阵：output_root 声明不重定向生成布局——产物落工程
    // gen/front/<profile>（报告 output_root 即真实布局根）。
    let _ = out_root;
    let real_root = std::path::PathBuf::from(&report.output_root);
    assert!(
        std::fs::read_dir(&real_root).expect("dir").count() > 0,
        "reported output root must hold real artifacts: {real_root:?}"
    );
    client.shutdown().expect("shutdown");
}

/// PLAN-816 T-05 R1（F-02/F-03）永久回归：InputAck→RunOk 合法交错 +
/// stdin 持有下的泵有界回收。三形态经真实子进程（探针 bin 宿主+消费
/// 者一体；测试持有其 stdin 管道）：control=非交互正控；held=stdin 持
/// 有不投输入（shutdown Success+线程数回落）；input=Launched 后注入 q
/// （InputAck 驻留、run 以 RunOk 终态、线程回落）。
#[test]
fn run_input_interleave_and_pump_reclaim_are_clean() {
    fn probe(mode: &str) -> (String, String, i32) {
        let mode = mode.to_string();
        // 门禁入口先构建 bin（镜像 t04-discovery-host 的先建后测形态）：
        // target/debug/t05-pump-probe.exe。
        let bin = std::env::current_exe()
            .expect("test exe")
            .parent()
            .and_then(|d| d.parent())
            .expect("debug dir")
            .join("t05-pump-probe.exe");
        assert!(bin.exists(), "probe bin missing — build it first: cargo build --manifest-path test/fixtures/modularity/tool-client-consumer/Cargo.toml");
        let mut child = std::process::Command::new(bin)
            .arg(&mode)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("spawn probe");
        let mut stderr = child.stderr.take().unwrap();
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        let mode_for_thread = mode.clone();
        std::thread::spawn(move || {
            let mode = &mode_for_thread;
            let mut buf = [0u8; 512];
            let mut acc = String::new();
            use std::io::Read;
            loop {
                match stderr.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        acc.push_str(&String::from_utf8_lossy(&buf[..n]));
                        if acc.contains("HOST_RUN_READY") && mode.as_str() == "input" {
                            let _ = tx.send(());
                            break;
                        }
                    }
                }
            }
        });
        if mode.as_str() == "input" {
            rx.recv_timeout(std::time::Duration::from_secs(5))
                .expect("host ready");
            std::thread::sleep(std::time::Duration::from_millis(50));
            use std::io::Write;
            let mut stdin = child.stdin.take().unwrap();
            stdin
                .write_all(
                    b"q
",
                )
                .unwrap();
            stdin.flush().unwrap();
            drop(stdin);
        } else {
            // held：stdin 保持打开（不投输入）——测试进程持有写端。
        }
        let out = child.wait_with_output().expect("wait");
        (
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::new(),
            out.status.code().unwrap_or(-1),
        )
    }
    for mode in ["control", "held", "input"] {
        let (stdout, _, code) = probe(mode);
        assert_eq!(code, 0, "[{mode}] stdout: {stdout}");
        assert!(
            stdout.contains("run_result=Ok(Ok(RunReport"),
            "[{mode}] run must complete with the work terminal: {stdout}"
        );
        assert!(
            stdout.contains("shutdown=Ok(Shutdown { kind: Success"),
            "[{mode}] {stdout}"
        );
        // F-816-T05-R2-03：探针单行 verdict（无 spawn 后新增持久线程 +
        // ≥3 条客户端线程已退）——进程级线程计数含负载相关 OS 瞬态，
        // 计数相等 oracle 在满载下假红（见探针内注记）。
        let verdict = stdout
            .lines()
            .find(|l| l.starts_with("PROBE_VERDICT"))
            .unwrap_or_else(|| panic!("[{mode}] probe verdict missing: {stdout}"));
        assert!(
            verdict.contains(" ok "),
            "[{mode}] pump/client threads must be reclaimed: {stdout}"
        );
        if mode == "input" {
            assert!(
                stdout.contains("receipts=1"),
                "[{mode}] InputAck parked: {stdout}"
            );
        }
    }
}

/// V-05 负控：未知协议版本在副作用前具名拒绝，流保持可用。
#[test]
fn version_rejected_before_side_effects_stream_alive() {
    let out_root = tempfile::tempdir().expect("tempdir");
    let project = tempfile::tempdir().expect("tempdir");
    let mut client = real_client();
    let reply = client
        .request_with_version(
            4242,
            ClientBody::ValidateProject {
                request: WireRequest(sample_request(project.path(), Some(out_root.path()))),
            },
        )
        .expect("host answers the rejection itself");
    match reply {
        Reply::Host(e) => assert_eq!(
            e.kind,
            ErrorKind::UnknownProtocolVersion {
                client: 4242,
                server: 1
            }
        ),
        other => panic!("expected host error, got {other:?}"),
    }
    assert_eq!(
        std::fs::read_dir(out_root.path()).expect("dir").count(),
        0,
        "rejection must have zero side effects"
    );
    // 拒绝后流仍可用。
    assert!(client.describe().is_ok());
    let shutdown = client.shutdown().expect("shutdown after rejection");
    assert_eq!(shutdown.kind, ExitKind::Success);
}

/// V-05 负控（缺能力）：能力表之外的操作具名拒绝、零副作用、流可用——
/// M815-A 未交付 generate，不得静默 fallback 或假装成功。
#[test]
fn missing_capability_rejected_without_fallback() {
    let out_root = tempfile::tempdir().expect("tempdir");
    let mut client = real_client();
    let reply = client
        .request_unknown_operation("generate")
        .expect("host answers unknown-operation itself");
    match reply {
        Reply::Host(e) => assert_eq!(
            e.kind,
            ErrorKind::UnknownOperation {
                operation: "generate".into()
            }
        ),
        other => panic!("expected host error, got {other:?}"),
    }
    assert_eq!(std::fs::read_dir(out_root.path()).expect("dir").count(), 0);
    assert!(client.describe().is_ok());
    client.shutdown().expect("shutdown");
}

/// V-05：cancel 诚实语义——已完成/不可取消操作回答 NotCancellable，
/// 不存在目标回答 InvalidField；无假成功。
#[test]
fn cancel_answers_honestly() {
    let mut client = real_client();
    client.describe().expect("describe first (served id=1)");
    let served = client.cancel_operation(1).expect("cancel answered");
    assert_eq!(
        served.kind,
        ErrorKind::NotCancellable {
            operation: "describe".into()
        }
    );
    let missing = client.cancel_operation(424242).expect("cancel answered");
    assert!(matches!(missing.kind, ErrorKind::InvalidField { .. }));
    client.shutdown().expect("shutdown");
}

/// V-02/V-05 寿命反例：请求超时 → owned 进程真停（收尸）+ reader 线程
/// join 后才返回 Timeout；此后连接具名 Broken，shutdown 迅速完成且非
/// Success（kill 发出≠已停；返回即已停）。
#[test]
fn request_timeout_reaps_owned_process_and_reader() {
    let mut spec = ToolClientSpec::new(te_child());
    spec.env = vec![
        ("TE_CHILD_MODE".into(), "sleep".into()),
        ("TE_CHILD_ARG1".into(), "30".into()),
    ];
    spec.request_timeout = Some(Duration::from_secs(2));
    let mut client = ToolClient::spawn(spec).expect("spawn te-child");
    let started = Instant::now();
    let err = client.describe().expect_err("host never answers");
    match &err {
        ClientError::Timeout { id: 1, limit } => assert_eq!(*limit, Duration::from_secs(2)),
        other => panic!("expected Timeout, got {other:?}"),
    }
    assert!(started.elapsed() < Duration::from_secs(5));
    // 连接已终态：后续请求具名拒绝。
    assert!(matches!(
        client.describe(),
        Err(ClientError::Broken(BrokenReason::TimedOut))
    ));
    // 已收尸：shutdown 立即完成（不会卡 10s 宽限），分类非 Success。
    let shutdown_started = Instant::now();
    let shutdown = client.shutdown().expect("shutdown after timeout");
    assert!(shutdown_started.elapsed() < Duration::from_secs(3));
    assert_ne!(shutdown.kind, ExitKind::Success);
}

/// V-02/V-05 寿命反例：孙进程随 owned 树真停——heartbeat 停止推进
/// （kill 发出≠已停的观察性证明）。
#[test]
fn grandchild_tree_stops_on_timeout() {
    let hb_dir = tempfile::tempdir().expect("tempdir");
    let hb = hb_dir.path().join("heartbeat.log");
    let mut spec = ToolClientSpec::new(te_child());
    spec.env = vec![
        ("TE_CHILD_MODE".into(), "grandchild".into()),
        ("TE_CHILD_ARG1".into(), "30".into()),
        (
            "TE_CHILD_ARG2".into(),
            hb.to_string_lossy().into_owned().into(),
        ),
    ];
    spec.request_timeout = Some(Duration::from_secs(2));
    let mut client = ToolClient::spawn(spec).expect("spawn te-child");
    // 等 grandchild 真跑起来再让超时触发（观察孙进程存活后取消——协议
    // 确定性关闭 assign 竞态窗口，同 T-02 反例）。
    let deadline = Instant::now() + Duration::from_secs(2);
    while std::fs::metadata(&hb).map(|m| m.len()).unwrap_or(0) == 0 {
        assert!(Instant::now() < deadline, "grandchild never started");
        std::thread::sleep(Duration::from_millis(50));
    }
    let err = client.describe().expect_err("host never answers");
    assert!(matches!(err, ClientError::Timeout { .. }), "{err:?}");
    let stopped_at = std::fs::metadata(&hb).map(|m| m.len()).unwrap_or(0);
    std::thread::sleep(Duration::from_millis(700));
    let after = std::fs::metadata(&hb).map(|m| m.len()).unwrap_or(0);
    assert_eq!(stopped_at, after, "grandchild must be dead with the tree");
}

/// V-02/V-05 负控：控制流出现非帧字节 → 具名 StreamViolation（不静默
/// 解析、不吞）；此后连接 Broken；宿主退出后 shutdown 正常完成。
#[test]
fn stream_pollution_is_named_violation() {
    let mut spec = ToolClientSpec::new(te_child());
    spec.env = vec![
        ("TE_CHILD_MODE".into(), "stdout-kib".into()),
        ("TE_CHILD_ARG1".into(), "1".into()),
        // 写完垃圾后保持存活，使客户端请求写入先于进程退出（时序确定）。
        ("TE_CHILD_ARG2".into(), "2".into()),
    ];
    spec.request_timeout = Some(Duration::from_secs(3));
    let mut client = ToolClient::spawn(spec).expect("spawn te-child");
    let err = client
        .describe()
        .expect_err("stdout carries non-frame bytes");
    assert!(
        matches!(err, ClientError::StreamViolation(_)),
        "expected named stream violation, got {err:?}"
    );
    assert!(matches!(
        client.describe(),
        Err(ClientError::Broken(BrokenReason::StreamViolated))
    ));
    client.shutdown().expect("shutdown after violation");
}

/// V-02/V-05：cwd/env 经客户端 spec 真实到达宿主进程；stdout 保持纯
/// （干净 EOF = StreamClosed 而非污染）；日志流承载身份证据。
#[test]
fn cwd_env_reach_host_and_logs_stay_separate() {
    let cwd = tempfile::tempdir().expect("tempdir");
    let mut spec = ToolClientSpec::new(te_child());
    spec.cwd = Some(cwd.path().to_path_buf());
    spec.env = vec![
        ("TE_CHILD_MODE".into(), "probe".into()),
        ("TE_CHILD_ARG1".into(), "AUTO_TOOL_CLIENT_PROBE".into()),
        ("AUTO_TOOL_CLIENT_PROBE".into(), "marker-✓-816".into()),
    ];
    let mut client = ToolClient::spawn(spec).expect("spawn te-child");
    // probe 模式：stderr 两行后 300ms 退出，stdout 全程为空 → 干净 EOF。
    let err = client.describe().expect_err("probe never answers");
    assert!(
        matches!(err, ClientError::StreamClosed { .. }),
        "expected clean EOF, got {err:?}"
    );
    let logs = client.logs();
    assert!(
        logs.contains(&format!("cwd {}", cwd.path().to_string_lossy())),
        "cwd must reach the host: {logs}"
    );
    assert!(
        logs.contains("env AUTO_TOOL_CLIENT_PROBE=marker-✓-816"),
        "{logs}"
    );
    let shutdown = client.shutdown().expect("shutdown");
    assert_eq!(format!("{:?}", shutdown.kind), "Success");
}

/// V-03 负控（预算）：超过帧预算的请求在写出前具名拒绝（TooLarge），
/// 不截断发送、不静默成功。
#[test]
fn oversize_request_rejected_before_io() {
    let mut client = real_client();
    let huge = "g".repeat(9 * 1024 * 1024);
    let err = client
        .request_unknown_operation(&huge)
        .expect_err("frame budget must reject the request");
    match &err {
        ClientError::SendFrame(frame_err) => {
            assert!(matches!(frame_err, FrameError::TooLarge { .. }));
        }
        other => panic!("expected SendFrame::TooLarge, got {other:?}"),
    }
    // 预算拒绝不污染流：宿主仍正常服务。
    assert!(client.describe().is_ok());
    client.shutdown().expect("shutdown");
}

/// F-816-R1-01：请求 deadline 覆盖**阻塞发送**——宿主不读 stdin、帧超
/// 管道缓冲时，250ms 截止以 Timeout 分类返回（终止 owned 树解除 writer
/// 阻塞），而非等到宿主 2s 自停后以 BrokenPipe 冒出。
#[test]
fn request_deadline_covers_blocked_send() {
    let mut spec = ToolClientSpec::new(te_child());
    spec.env = vec![
        ("TE_CHILD_MODE".into(), "sleep".into()),
        ("TE_CHILD_ARG1".into(), "2".into()),
    ];
    spec.request_timeout = Some(Duration::from_millis(250));
    let mut client = ToolClient::spawn(spec).expect("spawn te-child");
    let started = Instant::now();
    let err = client
        .request_unknown_operation(&"g".repeat(256 * 1024)) // 超管道缓冲、低于帧预算
        .expect_err("blocked send must hit the deadline");
    match &err {
        ClientError::Timeout { id: 1, limit } => {
            assert_eq!(*limit, Duration::from_millis(250));
        }
        other => panic!("expected Timeout, got {other:?}"),
    }
    assert!(
        started.elapsed() < Duration::from_millis(1200),
        "deadline must bound the blocked send (elapsed={:?})",
        started.elapsed()
    );
}

/// F-816-R1-02：宿主根进程秒退、持管道后代存活时，shutdown 在宽限内
/// 完成——根退出后 owned 后代先终止、管道关闭、reader 收尾；真实退出
/// 分类保留（Success），持管道后代随树真停（heartbeat 停止推进）。
#[test]
fn shutdown_bounds_pipe_holding_descendants() {
    let hb_dir = tempfile::tempdir().expect("tempdir");
    let hb = hb_dir.path().join("heartbeat.log");
    let mut spec = ToolClientSpec::new(te_child());
    spec.env = vec![
        ("TE_CHILD_MODE".into(), "orphan".into()),
        ("TE_CHILD_ARG1".into(), "3".into()),
        (
            "TE_CHILD_ARG2".into(),
            hb.to_string_lossy().into_owned().into(),
        ),
    ];
    spec.shutdown_timeout = Duration::from_millis(800);
    let client = ToolClient::spawn(spec).expect("spawn te-child");
    // 先证后代确已存活并持有管道(heartbeat 首跳),再走 shutdown——
    // 否则 shutdown 的树终止可能先于后代首跳,观察时序不确定。
    let deadline = Instant::now() + Duration::from_secs(2);
    while std::fs::metadata(&hb).map(|m| m.len()).unwrap_or(0) == 0 {
        assert!(Instant::now() < deadline, "descendant never started");
        std::thread::sleep(Duration::from_millis(50));
    }
    // 不发请求：直接走 shutdown——根秒退（真实分类 Success）后，
    // 持管道的 owned 后代先终止，宽限内完成收尾。
    let started = Instant::now();
    let shutdown = client.shutdown().expect("shutdown bounded");
    let elapsed = started.elapsed();
    assert_eq!(shutdown.kind, ExitKind::Success);
    assert!(
        elapsed < Duration::from_millis(2000),
        "shutdown must bound pipe-holding descendants (elapsed={elapsed:?})"
    );
    // 后代确已随树终止：heartbeat 停止推进。
    let stopped_at = std::fs::metadata(&hb).map(|m| m.len()).unwrap_or(0);
    std::thread::sleep(Duration::from_millis(600));
    let after = std::fs::metadata(&hb).map(|m| m.len()).unwrap_or(0);
    assert_eq!(stopped_at, after, "pipe-holding descendant must be dead");
}

/// F-816-R2-03：关闭阶段坏帧不被吞——宿主在 EOF 后写坏前缀再退出，
/// shutdown 必须返回具名 StreamViolation(TooLarge)，不得报 Success。
#[test]
fn close_phase_bad_frame_is_not_swallowed() {
    let mut spec = ToolClientSpec::new(te_child());
    spec.env = vec![
        ("TE_CHILD_MODE".into(), "proto-close".into()),
        ("TE_CHILD_ARG1".into(), "bad".into()),
    ];
    let mut client = ToolClient::spawn(spec).expect("spawn te-child");
    let reply = client
        .request_unknown_operation("probe")
        .expect("host answers the legal error envelope");
    assert!(matches!(reply, Reply::Host(_)), "{reply:?}");
    let err = client
        .shutdown()
        .expect_err("close-phase garbage must fail");
    match &err {
        ClientError::StreamViolation(FrameError::TooLarge { observed, .. }) => {
            assert_eq!(*observed, 0xffff_ffff);
        }
        other => panic!("expected StreamViolation(TooLarge), got {other:?}"),
    }
}

/// F-816-R2-03 对照：同一流程、宿主干净退出——shutdown 正常 Success。
#[test]
fn close_phase_clean_eof_control() {
    let mut spec = ToolClientSpec::new(te_child());
    spec.env = vec![
        ("TE_CHILD_MODE".into(), "proto-close".into()),
        ("TE_CHILD_ARG1".into(), "clean".into()),
    ];
    let mut client = ToolClient::spawn(spec).expect("spawn te-child");
    assert!(matches!(
        client.request_unknown_operation("probe"),
        Ok(Reply::Host(_))
    ));
    let shutdown = client.shutdown().expect("clean close");
    assert_eq!(shutdown.kind, ExitKind::Success);
}

/// F-816-R2-06：DirectChildOnly + 持管道非拥有后代——请求超时后客户端
/// 的自有 reader/writer 线程必须**真停**（有界返回，不带活线程）：
/// Timeout 在 ~deadline 返回（R2 反例实测 6.3s/线程存活；修复后 <
/// 1.5s），随后 shutdown 快速完成。
#[test]
fn abort_stops_own_threads_despite_unowned_holders() {
    let hb_dir = tempfile::tempdir().expect("tempdir");
    let hb = hb_dir.path().join("heartbeat.log");
    let mut spec = ToolClientSpec::new(te_child());
    spec.env = vec![
        ("TE_CHILD_MODE".into(), "orphan".into()),
        ("TE_CHILD_ARG1".into(), "4".into()),
        (
            "TE_CHILD_ARG2".into(),
            hb.to_string_lossy().into_owned().into(),
        ),
    ];
    spec.tree_scope = TreeScope::DirectChildOnly; // 非拥有后代保留（scope 合同）
    spec.request_timeout = Some(Duration::from_millis(250));
    let mut client = ToolClient::spawn(spec).expect("spawn te-child");
    let started = Instant::now();
    let err = client.describe().expect_err("no answer while pipes held");
    match &err {
        ClientError::Timeout { id: 1, limit } => assert_eq!(*limit, Duration::from_millis(250)),
        other => panic!("expected Timeout, got {other:?}"),
    }
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_millis(1500),
        "abort must stop own threads within bound (elapsed={elapsed:?})"
    );
    // 后续 shutdown 在已收尾的连接上快速完成;orphan 根进程以 0 退出,
    // 其真实分类(Success)保留——非拥有后代不在清理范围(scope 合同)。
    let shutdown_started = Instant::now();
    let shutdown = client.shutdown().expect("shutdown after abort");
    assert!(shutdown_started.elapsed() < Duration::from_secs(2));
    assert_eq!(shutdown.kind, ExitKind::Success);
}

/// F-816-R5 写端真停（客户端实机面，镜像 R5 探针）：DirectChildOnly +
/// 非拥有后代持有 stdin 读端且不读——256KiB 请求帧阻塞写。250ms 超时
/// 必须以 **Timeout** 分类有界返回（写线程经取消真停；非"带活线程的
/// PipesHeld"），随后 shutdown 快速完成。
#[test]
fn abort_stops_blocked_writer_for_real() {
    let hb_dir = tempfile::tempdir().expect("tempdir");
    let hb = hb_dir.path().join("heartbeat.log");
    let mut spec = ToolClientSpec::new(te_child());
    spec.env = vec![
        ("TE_CHILD_MODE".into(), "orphan".into()),
        ("TE_CHILD_ARG1".into(), "4".into()),
        (
            "TE_CHILD_ARG2".into(),
            hb.to_string_lossy().into_owned().into(),
        ),
    ];
    spec.tree_scope = TreeScope::DirectChildOnly; // 非拥有后代保留（scope 合同）
    spec.request_timeout = Some(Duration::from_millis(250));
    let mut client = ToolClient::spawn(spec).expect("spawn te-child");
    let started = Instant::now();
    let err = client
        .request_unknown_operation(&"g".repeat(256 * 1024)) // 超管道缓冲、低于帧预算
        .expect_err("blocked write must hit the deadline");
    let elapsed = started.elapsed();
    match &err {
        ClientError::Timeout { id: 1, limit } => assert_eq!(*limit, Duration::from_millis(250)),
        other => panic!("expected Timeout (writer truly stopped), got {other:?}"),
    }
    assert!(
        elapsed < Duration::from_millis(1500),
        "abort must be bounded with the writer truly stopped (elapsed={elapsed:?})"
    );
    // 非拥有后代保留（scope 合同）且随后 shutdown 快速完成。
    let shutdown_started = Instant::now();
    let shutdown = client.shutdown().expect("shutdown after abort");
    assert!(shutdown_started.elapsed() < Duration::from_secs(2));
    assert_eq!(shutdown.kind, ExitKind::Success);
}

// ============================================================================
// T-04 窄能力发现：discovery/版本/能力 + 缺件/缺 UI 负控（M815-A 面）
// ============================================================================

/// T-04 正控：按发现序解析到真实 auto-ui-tool，describe 身份/版本/能力齐。
#[test]
fn discovery_finds_real_tool_with_m815a_surface() {
    let mut spec = DiscoverySpec::auto_ui_tool();
    spec.workspace_root = Some(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../..")
            .canonicalize()
            .expect("workspace root"),
    );
    let found = discover(&spec).expect("discover real auto-ui-tool");
    assert!(found.path.exists());
    assert_eq!(found.descriptor.tool_name, "auto-ui-tool");
    assert_eq!(found.descriptor.protocol_version, PROTOCOL_VERSION);
    assert!(found.descriptor.offers("describe"));
    assert!(found.descriptor.offers("validate_project"));
    // M815-A 窄面：无 generate/build/run UI 能力。
    for ui_op in ["generate", "build", "run"] {
        assert!(
            !found.descriptor.offers(ui_op),
            "narrow tool must not claim UI op `{ui_op}`"
        );
    }
}

/// T-04 负控（缺件）：env 指向不存在路径 → 具名 EnvPathMissing，无回落。
#[test]
fn discovery_missing_env_path_is_named() {
    let mut spec = DiscoverySpec::auto_ui_tool();
    spec.env_key = "T04_MISSING_TOOL_BIN".into();
    std::env::set_var("T04_MISSING_TOOL_BIN", r"R:\definitely\not\a\tool.exe");
    let err = discover(&spec).expect_err("missing env path");
    assert!(
        matches!(err, DiscoveryError::EnvPathMissing { .. }),
        "got {err}"
    );
    std::env::remove_var("T04_MISSING_TOOL_BIN");
}

/// T-04 负控（缺件）：无 env、workspace 无该产物 → 具名 NotFound。
#[test]
fn discovery_not_found_is_named() {
    let mut spec = DiscoverySpec::auto_ui_tool();
    spec.env_key = "T04_NO_SUCH_ENV_KEY".into();
    spec.tool_name = "t04-no-such-tool".into();
    spec.workspace_root = Some(std::env::temp_dir().join("t04-empty-ws"));
    std::fs::create_dir_all(spec.workspace_root.as_ref().unwrap()).ok();
    let err = discover(&spec).expect_err("tool not found");
    assert!(matches!(err, DiscoveryError::NotFound { .. }), "got {err}");
}

/// T-04 负控（缺 UI 能力）：要求 generate 而窄工具不提供 → CapabilityMissing，
/// 且**不**静默 fallback。
#[test]
fn discovery_missing_ui_capability_is_named_no_fallback() {
    let mut spec = DiscoverySpec::auto_ui_tool();
    spec.workspace_root = Some(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../..")
            .canonicalize()
            .expect("workspace root"),
    );
    spec.required_operations = vec!["generate".into()];
    let err = discover(&spec).expect_err("narrow tool has no generate");
    match &err {
        DiscoveryError::CapabilityMissing { operation, .. } => {
            assert_eq!(operation, "generate");
        }
        other => panic!("expected CapabilityMissing, got {other}"),
    }
}

/// T-04 负控（版本）：要求错误协议版本 → VersionMismatch。
#[test]
fn discovery_protocol_version_mismatch_is_named() {
    let mut spec = DiscoverySpec::auto_ui_tool();
    spec.workspace_root = Some(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../..")
            .canonicalize()
            .expect("workspace root"),
    );
    spec.protocol_version = 999;
    let err = discover(&spec).expect_err("protocol 999 unsupported");
    assert!(
        matches!(err, DiscoveryError::VersionMismatch { .. }),
        "got {err}"
    );
}

// ---- R1-01：describe 成功后收尾失败不得当作已发现成功 ----
// （T04_HOST_MODE 为进程全局，四态串行于单测，避免并行污染。）

#[cfg(test)]
fn t04_host_bin() -> std::path::PathBuf {
    let exe = if cfg!(windows) {
        "t04-discovery-host.exe"
    } else {
        "t04-discovery-host"
    };
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/debug")
        .join(exe);
    assert!(
        path.exists(),
        "t04-discovery-host not found at {} — cargo build --manifest-path this fixture",
        path.display()
    );
    path
}

#[cfg(test)]
fn discover_via_host(
    mode: &str,
) -> Result<auto_tool_client::discovery::DiscoveredTool, DiscoveryError> {
    let mut spec = DiscoverySpec::auto_ui_tool();
    spec.env_key = format!("T04_HOST_{mode}");
    std::env::set_var(&spec.env_key, t04_host_bin());
    std::env::set_var("T04_HOST_MODE", mode);
    let r = discover(&spec);
    std::env::remove_var(&spec.env_key);
    std::env::remove_var("T04_HOST_MODE");
    r
}

/// R1-01：合法 describe 后四态收尾——干净 EOF 可接受；坏帧/exit1/exit7
/// 均 TeardownFailed 且不回落。
#[test]
fn discovery_teardown_failures_are_named_not_ok() {
    // 正控：干净 EOF / exit 0。
    let found = discover_via_host("good").expect("clean teardown accepted");
    assert_eq!(found.descriptor.tool_name, "auto-ui-tool");

    // 负控：EOF 后坏控制帧。
    let err = discover_via_host("tailbad").expect_err("bad frame after describe");
    assert!(
        matches!(err, DiscoveryError::TeardownFailed { .. }),
        "tailbad got {err}"
    );

    // 负控：exit 1（内部失败）。
    let err = discover_via_host("exit1").expect_err("exit 1 after describe");
    match &err {
        DiscoveryError::TeardownFailed { reason, .. } => {
            assert!(
                reason.contains('1') || reason.to_lowercase().contains("failure"),
                "exit1 reason={reason}"
            );
        }
        other => panic!("expected TeardownFailed, got {other}"),
    }

    // 负控：exit 7（异常码，须原样出现在 reason）。
    let err = discover_via_host("exit7").expect_err("exit 7 after describe");
    match &err {
        DiscoveryError::TeardownFailed { reason, .. } => {
            assert!(reason.contains('7'), "exit7 reason must carry 7: {reason}");
        }
        other => panic!("expected TeardownFailed, got {other}"),
    }
}

// ---- F-816-T05-R2-04：op 级执行选项协商（旧 v1 宿主负控 + 真工具广播）----

#[cfg(test)]
fn t05_gate_host_bin() -> std::path::PathBuf {
    let exe = if cfg!(windows) {
        "t05-option-gate-host.exe"
    } else {
        "t05-option-gate-host"
    };
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/debug")
        .join(exe);
    assert!(
        path.exists(),
        "t05-option-gate-host not found at {} — cargo build --manifest-path this fixture",
        path.display()
    );
    path
}

/// 旧 v1 宿主（describe 无 options 广播，与升级前完整工具同形）：携带
/// 执行选项的请求必须在**发送前**具名拒绝（零工作帧到达=零副作用）；
/// 同一宿主上无选项基础请求照常工作（旧工具不一刀切拒绝）。
#[test]
fn option_gate_rejects_unadvertised_options_before_side_effects() {
    let scratch = std::env::temp_dir().join(format!("p816-gate-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    let marker = scratch.join("leak.marker");

    let mut spec = ToolClientSpec::new(t05_gate_host_bin());
    spec.env = vec![("T05_GATE_MARKER".into(), marker.clone().into_os_string())];
    let mut client = ToolClient::spawn(spec).expect("spawn gate host");

    // 基线钉住：宿主 describe 为 v1 形（能力项全部无 options 广播）。
    let descriptor = client.describe().expect("describe from gate host");
    assert!(
        descriptor.capabilities.iter().all(|c| c.options.is_empty()),
        "gate host must look exactly like a pre-upgrade v1 tool"
    );

    let base = scratch.join("proj");
    std::fs::create_dir_all(base.join("src/front")).unwrap();
    let request = sample_request(&base, None);

    // 负控①：generate 携带 output——发送前具名拒绝。
    let opts = auto_tool_client::GenerateUiOpts {
        output: Some(scratch.join("out")),
        ..Default::default()
    };
    let err = client
        .generate_ui_opts(request.clone(), &opts)
        .expect_err("output option must be rejected against v1 host");
    match &err {
        ClientError::UnsupportedOption {
            option, operation, ..
        } => {
            assert_eq!(option, "output");
            assert_eq!(*operation, "generate_ui");
        }
        other => panic!("expected UnsupportedOption, got {other:?}"),
    }
    // 负控②：build 显式 strict。
    let err = client
        .build_ui_opts(request.clone(), Some(true))
        .expect_err("strict must be rejected against v1 host");
    match &err {
        ClientError::UnsupportedOption {
            option, operation, ..
        } => {
            assert_eq!(option, "strict");
            assert_eq!(*operation, "build_ui");
        }
        other => panic!("expected UnsupportedOption, got {other:?}"),
    }
    // 负控③：run 透传 args。
    let err = client
        .run_ui_opts(request.clone(), vec!["x".into()], false)
        .expect_err("run args must be rejected against v1 host");
    match &err {
        ClientError::UnsupportedOption {
            option, operation, ..
        } => {
            assert_eq!(option, "args");
            assert_eq!(*operation, "run_ui");
        }
        other => panic!("expected UnsupportedOption, got {other:?}"),
    }
    // 零副作用：宿主未收到任何携带选项的工作帧。
    assert!(
        !marker.exists(),
        "no option-carrying request may reach the host (side-effect window)"
    );
    // 正控：同一旧宿主上无选项基础请求照常成功。
    client
        .generate_ui(request)
        .expect("basic request still works against v1 host")
        .expect("basic generate succeeds");
    assert!(!marker.exists(), "basic request is not a leak");

    client.shutdown().expect("clean shutdown");
    let _ = std::fs::remove_dir_all(&scratch);
}

/// F-816-T05-R2-04 正控面：真实工具按操作广播兑现的执行选项（客户端
/// 协商依据——广告缺席即拒绝；此测试钉住广播本身不回退）。
#[test]
fn real_tool_advertises_op_options_for_negotiation() {
    let mut client = real_client();
    let descriptor = client.describe().expect("describe real tool");
    let generate = descriptor
        .capability("generate_ui")
        .expect("generate_ui capability");
    for option in ["output", "project", "delivery_root", "strict"] {
        assert!(
            generate.honors(option),
            "generate_ui must advertise {option}"
        );
    }
    assert!(descriptor
        .capability("build_ui")
        .expect("build_ui capability")
        .honors("strict"));
    assert!(descriptor
        .capability("run_ui")
        .expect("run_ui capability")
        .honors("args"));
}

// ---- F-816-T05-R2-02：生成报告绑定实际落点（真实工具进程 oracle）----

#[cfg(test)]
fn count_files_real(dir: &std::path::Path) -> u64 {
    fn walk(dir: &std::path::Path, n: &mut u64) {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let p = entry.path();
            if p.is_dir() {
                walk(&p, n);
            } else {
                *n += 1;
            }
        }
    }
    let mut n = 0u64;
    walk(dir, &mut n);
    n
}

#[cfg(test)]
fn render_project(base: &std::path::Path, render: &str) -> GenerationRequest {
    std::fs::create_dir_all(base.join("src").join("front")).expect("front dir");
    std::fs::write(
        base.join("src").join("front").join("app.at"),
        "widget App {
    view {
        col {
            text \"report oracle\" {}
        }
    }
}
",
    )
    .expect("app.at");
    let pac = base.join("pac.at");
    let text = format!("name: \"report oracle\"\nrender: \"{render}\"\n");
    std::fs::write(&pac, &text).expect("pac");
    GenerationRequest::from_descriptor(
        &parse_document(&text, SourceInfo::from_path(&pac)).expect("parse"),
    )
}

/// F-816-T05-R2-02：显式 output 重定向下，报告 `output_root` = 重定向
/// 目录本身、`files_written` = 对真实文件的走查计数（错误根/默认根不
/// 被报成产物）。覆盖可重定向四 profile 同族（rust/jet/ark/vscode）。
#[test]
fn generate_report_binds_redirected_output_for_all_profiles() {
    let mut client = real_client();
    for profile in ["rust", "jet", "ark", "vscode"] {
        let project = tempfile::tempdir().expect("project tempdir");
        let out = tempfile::tempdir().expect("output tempdir");
        let request = render_project(project.path(), profile);
        let opts = auto_tool_client::GenerateUiOpts {
            output: Some(out.path().to_path_buf()),
            ..Default::default()
        };
        let report = client
            .generate_ui_opts(request, &opts)
            .unwrap_or_else(|e| panic!("{profile}: client error {e}"))
            .unwrap_or_else(|e| panic!("{profile}: host rejected {e}"));
        assert_eq!(
            std::path::Path::new(&report.output_root),
            out.path(),
            "{profile}: report output_root must be the redirect dir"
        );
        let real = count_files_real(out.path());
        assert!(
            real > 0,
            "{profile}: real artifacts must exist under redirect"
        );
        assert_eq!(
            report.files_written, real,
            "{profile}: files_written must equal the real walk count"
        );
    }
    client.shutdown().expect("shutdown");
}

/// F-816-T05-R2-02（delivery 正控）：792 显式交付的报告同样绑定
/// delivery 根与真实计数，delivery-manifest 真实存在。
#[test]
fn generate_report_binds_delivery_root() {
    let project = tempfile::tempdir().expect("project tempdir");
    let delivery = tempfile::tempdir().expect("delivery tempdir");
    let request = render_project(project.path(), "rust");
    let opts = auto_tool_client::GenerateUiOpts {
        delivery_root: Some(delivery.path().to_path_buf()),
        ..Default::default()
    };
    let mut client = real_client();
    let report = client
        .generate_ui_opts(request, &opts)
        .expect("client round trip")
        .expect("delivery generates");
    assert_eq!(
        std::path::Path::new(&report.output_root),
        delivery.path(),
        "delivery report root must be the delivery dir"
    );
    let real = count_files_real(delivery.path());
    assert!(real > 0, "delivery bundle must hold real files");
    assert_eq!(report.files_written, real);
    assert!(
        delivery.path().join("delivery-manifest.json").exists(),
        "delivery manifest must exist"
    );
    client.shutdown().expect("shutdown");
}

/// F-816-T05-R2-02（错误根不被报成成功产物）：vue 携带显式 output 是
/// 具名拒绝（内层 ToolError），不返回任何成功报告。
#[test]
fn vue_explicit_output_is_rejected_not_reported() {
    let project = tempfile::tempdir().expect("project tempdir");
    let out = tempfile::tempdir().expect("output tempdir");
    let request = render_project(project.path(), "vue");
    let opts = auto_tool_client::GenerateUiOpts {
        output: Some(out.path().to_path_buf()),
        ..Default::default()
    };
    let mut client = real_client();
    let err = client
        .generate_ui_opts(request, &opts)
        .expect("client round trip")
        .expect_err("vue+output must be rejected by the host");
    assert!(err.to_string().contains("output"), "{err}");
    client.shutdown().expect("shutdown");
}

/// F-816-T05-R2-04 端到端核实（relay-lie 宿主）：describe 广播选项、
/// 回"成功"但落点在默认布局——同构旧 wire 中间人剥掉请求选项。客户端
/// 落点核对必须具名 `OptionIgnored`（外层 ClientError），不把被丢选项
/// 的产物当成功。
#[test]
fn option_ignored_by_lying_relay_is_named_failure() {
    let scratch = std::env::temp_dir().join(format!("p816-lie-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    let marker = scratch.join("leak.marker");

    let mut spec = ToolClientSpec::new(t05_gate_host_bin());
    spec.env = vec![
        ("T05_GATE_MARKER".into(), marker.clone().into_os_string()),
        ("T05_GATE_ADVERTISE".into(), "1".into()),
    ];
    let mut client = ToolClient::spawn(spec).expect("spawn relay-lie host");

    // 基线：该宿主 describe 广播 generate_ui 选项（协商放行的前提）。
    let descriptor = client.describe().expect("describe");
    assert!(
        descriptor
            .capability("generate_ui")
            .expect("generate_ui")
            .honors("output"),
        "relay-lie host must advertise output"
    );

    let base = scratch.join("proj");
    std::fs::create_dir_all(base.join("src/front")).unwrap();
    let request = sample_request(&base, None);
    let opts = auto_tool_client::GenerateUiOpts {
        output: Some(scratch.join("requested-out")),
        ..Default::default()
    };
    match client.generate_ui_opts(request, &opts) {
        Err(ClientError::OptionIgnored {
            option, reported, ..
        }) => {
            assert_eq!(option, "output");
            assert_eq!(reported, "C:/lie/default-layout-root");
        }
        other => panic!("expected OptionIgnored, got {other:?}"),
    }
    client.shutdown().expect("shutdown");
    let _ = std::fs::remove_dir_all(&scratch);
}

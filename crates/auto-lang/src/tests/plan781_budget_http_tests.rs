//! PLAN-781 T-16/T-22: HTTP 请求级累计步预算真 TCP e2e（合同 =
//! docs/plans/reports/781-budget-decision.md D-2 入口矩阵 / §2 V-14/V-18；
//! W-04 生命周期 oracle = F-781-R04 修正要求）。
//!
//! `http_e2e_plan781` 族挂 `test-http-e2e`（cargo th 档；nextest 每测试
//! 一进程——env 置换天然隔离）。服务器 fixture 同 plan730 `start_server`
//! 形态（真 TCP、crate::run 自动起服）；预算经 `AUTO_HTTP_STEP_BUDGET`
//! env 注入（AutoVM 构造期快照 → LegacyHttp 累计档）。
//!
//! 覆盖：低预算 = 指名资源拒绝（500 + "per-request cumulative budget"）
//! 且服务继续服务后续请求（V-14/V-18 单失败终态）；高预算同 handler 完成
//! （V-14 正例）。closure 入口（V-20）在 plan781_budget_tests 引擎级覆盖
//! （plan705 合成 fn-ref closure 模式）。
//!
//! W-04（T-22 / AC-17 / F-781-R04）：LegacyHttp 长忙请求的三路生命周期
//! oracle——`/api/slowwrite` 以 `io.write_text_async` 为每次心跳的真 park
//! 点（副作用文件落盘 + live-op/COMPLETION_NOTIFY 事件驱动恢复），entered
//! 握手确认执行已开始；deadline/断连/服务关闭三路各自断言：取消后零新
//! 写入（心跳冻结、至多一个在飞写落盘）、无迟到成功（done 永不出现）、
//! 单终态（客户端恰一次 503/连接终结；scope 状态机幂等收敛）、task/scope/
//! live-op 回收（`live_scope_count`/`live_op_count` 回基线）与生命期许可
//! 归还（容量 1 下 cancel 前 ping 503、cancel 后 ping 200）。R04 修复轮
//! 补 CPU 段形态（`/api/cpuloop`：同步 entered/late + 4M 纯 CPU 环、无
//! park 边界）——engine 驱动环每 HTTP_CANCEL_PROBE_EVERY 步协作式取消
//! 探测，deadline 负/正对照 + RST/关闭组合，终态后 late 副作用在超全程
//! 观测窗内缺席。旧"断连仅查 ping 存活"短路径证据已并入 W-04 断连测
//! （ping 存活在许可归还断言中承载）；plan705/707 族的 scope 机制单测
//! 仍持有失效队列/半关闭边界，本族持有 LegacyHttp policy 长忙路径的端
//! 到端绑定。

#[cfg(feature = "test-http-e2e")]
mod budget_e2e {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::path::Path;
    use std::time::{Duration, Instant};

    /// 同 plan730 start_server 形态：编译+运行 .at（#[api] 路由自动起服，
    /// 真 TCP），失败载荷有界上报。
    fn start_server(code: &str, port: u16) -> u16 {
        crate::vm::ffi::stdlib::clear_http_routes();
        std::env::set_var("AUTO_HTTP_PORT", port.to_string());
        let code = code.to_string();
        let outcome_rx = crate::vm::ffi::http_server::spawn_outcome_reporter(
            8 * 1024 * 1024,
            "crate::run",
            move || crate::run(&code),
        );
        for _ in 0..50 {
            if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                return port;
            }
            if let Ok(outcome) = outcome_rx.try_recv() {
                panic!("test server failed to start on port {port}: {outcome}");
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        panic!("test server never accepted connections on port {port}");
    }

    const PROGRAM: &str = r#"
#[api(method = "GET", path = "/api/heavy")]
fn heavy() str {
    var acc int = 0
    var i int = 0
    while i < 200000 {
        acc = acc + 1
        i = i + 1
    }
    return "done"
}

#[api(method = "GET", path = "/api/ping")]
fn ping() str {
    return "pong"
}
"#;

    /// W-04 slowwrite：entered 握手 → N 次心跳副作用写（每次
    /// `io.write_text_async` = 真 park 点，LegacyHttp policy 下跨多段累计）
    /// → done 迟到成功标记。正常完成需远超各取消路的触发时刻。
    fn slowwrite_program(dir: &str, iterations: usize) -> String {
        format!(
            r#"
#[api(method = "GET", path = "/api/slowwrite")]
fn slowwrite() str {{
    io.write_text_async("{dir}/entered", "1")
    var i int = 0
    while i < {iterations} {{
        io.write_text_async("{dir}/hb-" + i.to_string(), "x")
        i = i + 1
    }}
    io.write_text_async("{dir}/done", "1")
    return "slowwrite-done"
}}

#[api(method = "GET", path = "/api/ping")]
fn ping() str {{
    return "pong"
}}
"#
        )
    }

    fn http_get(port: u16, path: &str) -> (u16, String) {
        let mut s = TcpStream::connect(("127.0.0.1", port)).expect("connect");
        s.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
        write!(
            s,
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        let mut buf = String::new();
        s.read_to_string(&mut buf).unwrap_or_default();
        let status = buf
            .split_whitespace()
            .nth(1)
            .and_then(|c| c.parse::<u16>().ok())
            .unwrap_or(0);
        (status, buf)
    }

    fn free_port() -> u16 {
        std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    // ------------------------------------------------------------------
    // W-04 helpers：副作用文件观测（entered 握手 / 心跳计数 / 冻结判定）
    // ------------------------------------------------------------------

    fn hb_count(dir: &Path) -> usize {
        std::fs::read_dir(dir)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .filter(|e| e.file_name().to_string_lossy().starts_with("hb-"))
                    .count()
            })
            .unwrap_or(0)
    }

    /// entered 握手：确认 handler 执行确已开始（而非只发出了请求），
    /// 且心跳副作用在持续产生（min_hbs 由 0 轮询到 ≥min 证增长）。
    fn wait_handshake(dir: &Path, min_hbs: usize, timeout: Duration) -> usize {
        let deadline = Instant::now() + timeout;
        loop {
            if dir.join("entered").exists() {
                let n = hb_count(dir);
                if n >= min_hbs {
                    return n;
                }
            }
            assert!(
                Instant::now() < deadline,
                "slowwrite never entered/hb-progressed (entered={} hbs={})",
                dir.join("entered").exists(),
                hb_count(dir)
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// 取消后零新写入判定：心跳数稳定（相邻采样不变）即冻结；取消未生效
    /// 时持续增长（或跑满全部心跳+done），由调用方的 settled<total 与
    /// done 断言兜底判红。返回冻结后的最终计数。
    fn wait_writes_settled(dir: &Path, at_cancel: usize, total: usize) -> usize {
        let mut last = at_cancel;
        for _ in 0..40 {
            std::thread::sleep(Duration::from_millis(400));
            let now = hb_count(dir);
            if now == last {
                return now;
            }
            last = now;
        }
        panic!(
            "heartbeat writes never settled after cancellation \
             (last={last}, total={total}) — execution continued past cancel"
        );
    }

    // ------------------------------------------------------------------
    // W-04 CPU 段形态（R04 修复轮，复审反例原样）：同步 entered → 4M 次
    // 纯 CPU 环（无 park 边界，~32M 指令）→ 同步 late → done。修复=
    // engine 驱动环每 HTTP_CANCEL_PROBE_EVERY 步的协作式取消探测（安
    // 全指令边界中止段）；终态后的 late 副作用窗口以超过未取消全程
    //（~6s）的观测窗证明缺席。
    // ------------------------------------------------------------------

    fn cpuloop_program(dir: &str) -> String {
        format!(
            r#"
#[api(method = "GET", path = "/api/cpuloop")]
fn cpuloop() str {{
    File.write_text("{dir}/entered", "1")
    var acc int = 0
    var i int = 0
    while i < 4000000 {{
        acc = acc + i
        i = i + 1
    }}
    File.write_text("{dir}/late", "1")
    return "done"
}}

#[api(method = "GET", path = "/api/ping")]
fn ping() str {{
    return "pong"
}}
"#
        )
    }

    fn wait_entered(dir: &Path, timeout: Duration) {
        let deadline = Instant::now() + timeout;
        while !dir.join("entered").exists() {
            assert!(
                Instant::now() < deadline,
                "cpuloop handler never entered execution"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// late 副作用缺席证明：观测窗（10s）须超过未取消时的全程耗时
    /// （~6s）——取消失效时 late 必在窗内落盘（复审反例 5.9s 实测）。
    fn assert_late_never_written(dir: &Path, ctx: &str) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            assert!(
                !dir.join("late").exists(),
                "{ctx}: late side-effect written after terminal — \
                 cooperative cancel failed (CPU segment ran past cancel)"
            );
            std::thread::sleep(Duration::from_millis(200));
        }
    }

    fn slowwrite_dir() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let s = dir.path().to_string_lossy().replace('\\', "/");
        (dir, s)
    }

    /// W-04 共同收尾：心跳冻结于取消时刻附近（至多一个在飞写落盘的
    /// 探测余量）、远未跑满（取消确实生效于中途）、done 永不出现（无
    /// 迟到成功）、scope/live-op 回基线（task/scope/许可归还）。
    fn assert_cancelled_clean(
        dir: &Path,
        at_cancel: usize,
        total: usize,
        op_baseline: usize,
        ctx: &str,
    ) {
        let settled = wait_writes_settled(dir, at_cancel, total);
        assert!(
            settled < total,
            "{ctx}: cancelled run must not reach {total} heartbeats (got {settled})"
        );
        assert!(
            settled <= at_cancel + 200,
            "{ctx}: writes continued far past cancel point \
             (at_cancel={at_cancel}, settled={settled})"
        );
        assert!(
            !dir.join("done").exists(),
            "{ctx}: late success (done) must never be written after cancellation"
        );
        std::thread::sleep(Duration::from_millis(300));
        assert_eq!(
            crate::vm::ffi::async_http::live_op_count(),
            op_baseline,
            "{ctx}: live-op must return to baseline (task/scope reclaimed)"
        );
        assert_eq!(
            crate::vm::ffi::http_server::live_scope_count(),
            0,
            "{ctx}: scope map must be empty (single terminal reached, permit released)"
        );
    }

    /// V-14 负例 + V-18：低预算 = 指名资源拒绝（非解析错/空成功）+ 一次
    /// 失败终态后服务继续服务后续请求。生命周期深度（取消后零写入/回收）
    /// 由下方 W-04 三路持有，本测只持预算耗尽终态本身。
    #[test]
    fn http_e2e_plan781_budget_low_rejects_named_and_server_continues() {
        let port = free_port();
        std::env::set_var("AUTO_HTTP_STEP_BUDGET", "1000");
        start_server(PROGRAM, port);
        let (status, body) = http_get(port, "/api/heavy");
        assert_eq!(status, 500, "{body}");
        assert!(
            body.contains("per-request cumulative budget")
                && body.contains("AUTO_HTTP_STEP_BUDGET"),
            "named resource rejection required: {body}"
        );
        // 单失败终态：服务器继续服务（轻路由不受污染）。
        let (status2, body2) = http_get(port, "/api/ping");
        assert_eq!(status2, 200, "{body2}");
        assert!(body2.contains("pong"), "{body2}");
        // 再次拒绝仍指名（重复失败不变形）。
        let (status3, body3) = http_get(port, "/api/heavy");
        assert_eq!(status3, 500, "{body3}");
        assert!(body3.contains("per-request cumulative budget"), "{body3}");
    }

    /// V-14 正例：宽预算下同一 handler 完成（相同有限计算分别成功/受拒
    /// 的“成功”半边；低预算半边由上一测持有）。
    #[test]
    fn http_e2e_plan781_budget_generous_completes() {
        let port = free_port();
        std::env::set_var("AUTO_HTTP_STEP_BUDGET", "50000000");
        start_server(PROGRAM, port);
        let (status, body) = http_get(port, "/api/heavy");
        assert_eq!(status, 200, "{body}");
        assert!(body.contains("done"), "{body}");
    }

    /// W-04 路 1/3 —— deadline（V-18）：LegacyHttp policy 宽预算下长忙
    /// slowwrite 在请求 deadline（600ms）处收口：客户端恰一次 503 终态
    /// （远早于 20000 心跳的自然完成），取消后零新写入、无迟到成功、
    /// scope/live-op 回基线。预算非本路终态（400M 远不触），绑定的是
    /// “policy 生效下的长忙请求被 deadline 取消”这一生命周期面。
    #[test]
    fn http_e2e_plan781_w04_deadline_cancels_writes_single_terminal_reclaims() {
        let (dir, d) = slowwrite_dir();
        let port = free_port();
        std::env::set_var("AUTO_HTTP_STEP_BUDGET", "400000000");
        std::env::set_var("AUTO_HTTP_REQUEST_TIMEOUT_MS", "600");
        let total = 20000;
        start_server(&slowwrite_program(&d, total), port);
        let op_baseline = crate::vm::ffi::async_http::live_op_count();
        let t0 = Instant::now();
        let (status, body) = http_get(port, "/api/slowwrite");
        let elapsed = t0.elapsed();
        // 单终态：恰一次 503（deadline 桥臂与 owner abort 竞速，双路径
        // 幂等——客户端只见一次应答）。
        assert_eq!(status, 503, "deadline must yield single 503 terminal: {body}");
        assert!(
            body.contains("error") || body.contains("cancelled"),
            "503 terminal carries error form: {body}"
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "terminal must be deadline-bound (~600ms), not completion-bound: {elapsed:?}"
        );
        // entered 握手：执行确已开始后才被取消（非队列期拒绝）。
        assert!(
            dir.path().join("entered").exists(),
            "handler must have entered execution before deadline terminal"
        );
        let at_cancel = hb_count(dir.path());
        assert_cancelled_clean(dir.path(), at_cancel, total, op_baseline, "deadline");
    }

    /// W-04 路 2/3 —— 断连（V-18）：entered 握手 + 心跳持续后客户端以
    /// RST 中断（SO_LINGER=0 + close——网络中断/进程被杀的传输形态）：
    /// 连接任务终结 → conn 名下 scope 取消 → 心跳冻结、done 永不出现、
    /// scope/live-op 回基线；生命期许可容量 1 下——cancel 前 ping 503
    /// （许可被长忙请求持有）、cancel 后 ping 200（许可归还 + 服务存活，
    /// 旧“仅 ping 存活”冒烟并入此断言）。
    ///
    /// 边界（plan705 §5.4 半关闭契约，W-04 实测钉死）：FIN-only 的全关
    /// （drop 未设 linger）与 shutdown(Write) 半关闭在传输层不可区分——
    /// hyper `half_close(true)` 下服务端继续服务直至响应写失败或请求
    /// deadline（30s 默认）收口；该形态的迟到执行受 deadline 上界约束，
    /// 由 deadline 路与 plan705 T-05(4) 分别持有，不在本路断言取消。
    #[test]
    fn http_e2e_plan781_w04_client_disconnect_stops_writes_and_returns_permit() {
        let (dir, d) = slowwrite_dir();
        let port = free_port();
        std::env::set_var("AUTO_HTTP_STEP_BUDGET", "400000000");
        std::env::set_var("AUTO_HTTP_REQUEST_TIMEOUT_MS", "30000");
        std::env::set_var("AUTO_HTTP_MAX_INFLIGHT", "1");
        let total = 20000;
        start_server(&slowwrite_program(&d, total), port);
        let op_baseline = crate::vm::ffi::async_http::live_op_count();
        let mut s = TcpStream::connect(("127.0.0.1", port)).expect("connect");
        write!(
            &mut s,
            "GET /api/slowwrite HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        // entered 握手：确认执行已开始且心跳在持续产生。
        wait_handshake(dir.path(), 3, Duration::from_secs(15));
        // 许可被在途长忙请求持有：新请求 503（零分配零排队）。
        let (held, held_body) = http_get(port, "/api/ping");
        assert_eq!(
            held, 503,
            "life permit must be held by in-flight slowwrite: {held_body}"
        );
        let at_cancel = hb_count(dir.path());
        // 真断连 = RST（SO_LINGER=0 + close）。FIN-only 全关与半关闭
        // （shutdown(Write)）传输层同形——half_close 契约下不判取消，
        // 迟到执行由请求 deadline 上界约束（见函数文档）。
        socket2::SockRef::from(&s)
            .set_linger(Some(Duration::ZERO))
            .expect("SO_LINGER(0) for RST abort");
        drop(s);
        assert_cancelled_clean(dir.path(), at_cancel, total, op_baseline, "disconnect");
        // 许可归还：同一容量 1 下新请求必须重新可得且服务存活。
        let (after, after_body) = http_get(port, "/api/ping");
        assert_eq!(
            after, 200,
            "permit must be reclaimed after disconnect-cancel (server alive): {after_body}"
        );
        assert!(after_body.contains("pong"), "{after_body}");
    }

    /// W-04 路 3/3 —— 服务关闭（V-18）：排水窗（300ms）<< 自然完成时长；
    /// 优雅关停触发后 parked 心跳被取消：客户端单终态（503 或连接终结
    /// ——两者同义=请求未完成、等待已废弃），心跳冻结、无迟到成功、
    /// scope/live-op 回基线。
    #[test]
    fn http_e2e_plan781_w04_shutdown_drain_cancels_writes_and_reclaims() {
        let (dir, d) = slowwrite_dir();
        let port = free_port();
        std::env::set_var("AUTO_HTTP_STEP_BUDGET", "400000000");
        std::env::set_var("AUTO_HTTP_REQUEST_TIMEOUT_MS", "30000");
        std::env::set_var("AUTO_HTTP_SHUTDOWN_DRAIN_MS", "300");
        let total = 20000;
        start_server(&slowwrite_program(&d, total), port);
        let op_baseline = crate::vm::ffi::async_http::live_op_count();
        let client = std::thread::spawn(move || http_get(port, "/api/slowwrite"));
        wait_handshake(dir.path(), 3, Duration::from_secs(15));
        assert!(
            crate::vm::ffi::http_server::test_trigger_shutdown(),
            "shutdown trigger must be registered"
        );
        let (status, body) = client.join().expect("client thread");
        assert!(
            status == 503 || status == 0,
            "shutdown terminal must be 503 or connection close: {status} {body}"
        );
        let at_cancel = hb_count(dir.path());
        assert_cancelled_clean(dir.path(), at_cancel, total, op_baseline, "shutdown");
    }

    /// W-04 CPU 段负例（R04 修复轮；复审反例原样）：deadline=200ms 下
    /// 纯 CPU 4M 环（无 park 边界）——503 终态后 late 副作用必须在超过
    /// 未取消全程的观测窗内保持缺席（修复前 5.9s 时 late 落盘）；scope/
    /// live-op 回收。entered 证明执行确已开始后才被取消。
    #[test]
    fn http_e2e_plan781_w04_cpu_deadline_cancels_before_late_write() {
        let (dir, d) = slowwrite_dir();
        let port = free_port();
        std::env::set_var("AUTO_HTTP_STEP_BUDGET", "400000000");
        std::env::set_var("AUTO_HTTP_REQUEST_TIMEOUT_MS", "200");
        start_server(&cpuloop_program(&d), port);
        let op_baseline = crate::vm::ffi::async_http::live_op_count();
        let t0 = Instant::now();
        let (status, body) = http_get(port, "/api/cpuloop");
        let elapsed = t0.elapsed();
        assert_eq!(status, 503, "deadline must yield 503 terminal: {body}");
        assert!(
            elapsed < Duration::from_secs(3),
            "terminal must be deadline-bound (~200ms): {elapsed:?}"
        );
        assert!(
            dir.path().join("entered").exists(),
            "handler must have entered execution before terminal"
        );
        assert_late_never_written(dir.path(), "cpu-deadline");
        std::thread::sleep(Duration::from_millis(300));
        assert_eq!(
            crate::vm::ffi::async_http::live_op_count(),
            op_baseline,
            "live-op must return to baseline"
        );
        assert_eq!(
            crate::vm::ffi::http_server::live_scope_count(),
            0,
            "scope map must be empty"
        );
    }

    /// W-04 CPU 段正对照（R04 修复轮）：deadline=30s 下同一有限计算正常
    /// 完成（200 + late 在场）——取消检查不误伤合法长忙请求（复审控制
    /// 行 2 的镜像）。
    #[test]
    fn http_e2e_plan781_w04_cpu_generous_deadline_completes() {
        let (dir, d) = slowwrite_dir();
        let port = free_port();
        std::env::set_var("AUTO_HTTP_STEP_BUDGET", "400000000");
        std::env::set_var("AUTO_HTTP_REQUEST_TIMEOUT_MS", "30000");
        start_server(&cpuloop_program(&d), port);
        let (status, body) = http_get(port, "/api/cpuloop");
        assert_eq!(status, 200, "{body}");
        assert!(body.contains("done"), "{body}");
        assert!(
            dir.path().join("late").exists(),
            "positive control: full CPU run must write late"
        );
    }

    /// W-04 CPU 段 + RST 组合（R04 修复轮）：deadline 不参与，纯 CPU 段
    /// 中途 RST 断连——协作取消使 late 缺席；容量 1 许可两态（持有时
    /// ping 503、归还后 ping 200）。
    #[test]
    fn http_e2e_plan781_w04_cpu_rst_disconnect_no_late_write() {
        let (dir, d) = slowwrite_dir();
        let port = free_port();
        std::env::set_var("AUTO_HTTP_STEP_BUDGET", "400000000");
        std::env::set_var("AUTO_HTTP_REQUEST_TIMEOUT_MS", "30000");
        std::env::set_var("AUTO_HTTP_MAX_INFLIGHT", "1");
        start_server(&cpuloop_program(&d), port);
        let op_baseline = crate::vm::ffi::async_http::live_op_count();
        let mut s = TcpStream::connect(("127.0.0.1", port)).expect("connect");
        write!(
            &mut s,
            "GET /api/cpuloop HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        wait_entered(dir.path(), Duration::from_secs(15));
        let (held, held_body) = http_get(port, "/api/ping");
        assert_eq!(
            held, 503,
            "life permit must be held by in-flight cpuloop: {held_body}"
        );
        socket2::SockRef::from(&s)
            .set_linger(Some(Duration::ZERO))
            .expect("SO_LINGER(0) for RST abort");
        drop(s);
        assert_late_never_written(dir.path(), "cpu-rst");
        std::thread::sleep(Duration::from_millis(300));
        assert_eq!(
            crate::vm::ffi::async_http::live_op_count(),
            op_baseline,
            "live-op must return to baseline"
        );
        assert_eq!(
            crate::vm::ffi::http_server::live_scope_count(),
            0,
            "scope map must be empty"
        );
        let (after, after_body) = http_get(port, "/api/ping");
        assert_eq!(
            after, 200,
            "permit must be reclaimed after RST cancel (server alive): {after_body}"
        );
        assert!(after_body.contains("pong"), "{after_body}");
    }

    /// W-04 CPU 段 + 服务关闭组合（R04 修复轮）：排水窗 300ms 在 CPU 段
    /// 中途触发——单终态（503/连接终结）+ late 缺席 + scope/live-op 回
    /// 基线。
    #[test]
    fn http_e2e_plan781_w04_cpu_shutdown_no_late_write() {
        let (dir, d) = slowwrite_dir();
        let port = free_port();
        std::env::set_var("AUTO_HTTP_STEP_BUDGET", "400000000");
        std::env::set_var("AUTO_HTTP_REQUEST_TIMEOUT_MS", "30000");
        std::env::set_var("AUTO_HTTP_SHUTDOWN_DRAIN_MS", "300");
        start_server(&cpuloop_program(&d), port);
        let op_baseline = crate::vm::ffi::async_http::live_op_count();
        let client = std::thread::spawn(move || http_get(port, "/api/cpuloop"));
        wait_entered(dir.path(), Duration::from_secs(15));
        assert!(
            crate::vm::ffi::http_server::test_trigger_shutdown(),
            "shutdown trigger must be registered"
        );
        let (status, body) = client.join().expect("client thread");
        assert!(
            status == 503 || status == 0,
            "shutdown terminal must be 503 or connection close: {status} {body}"
        );
        assert_late_never_written(dir.path(), "cpu-shutdown");
        std::thread::sleep(Duration::from_millis(300));
        assert_eq!(
            crate::vm::ffi::async_http::live_op_count(),
            op_baseline,
            "live-op must return to baseline"
        );
        assert_eq!(
            crate::vm::ffi::http_server::live_scope_count(),
            0,
            "scope map must be empty"
        );
    }
}

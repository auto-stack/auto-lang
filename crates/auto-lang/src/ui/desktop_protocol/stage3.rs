// Plan 480 S4 —— Stage 3 多 App 共享 host 引擎 + 压测 harness。
//
// [`BrokerClient`]：一条孵化连接的宿主侧全部状态（端点 + 双缓冲表面 +
// shm 段 + wid 映射）——驻留 [`DesktopSession`](crate::ui::session::
// DesktopSession) 的 `broker_clients` 表，宿主因此可以**并发**承载 N 个
// child（`HostEndpoint` 单 client 的"多 App 并发归 Stage 3"正是本计划
// 兑现的预留）。attach / pump / 指针路由的作用逻辑在 session.rs 的
// `broker_*` 方法族（与 `host::ProtocolHost::handle` 动作臂同构；
// ProtocolHost 单 client 机件与 Stage 1/2 测试不动）。
//
// 压测 harness（本文件测试）：父进程 = 桌面侧（enable_broker + 注册表
// resolver + attach/pump 泵）；子进程 = **测试二进制 re-exec**
// （`stage3_child_body`，env 注入 broker 管道名与 app 名）经
// `request_incubation` 孵化 + `ClientPump::run` 真协议主循环。
// N=3/5 child 全 Active → 逐 App 点击帧递增 → 30s 稳定存活。

use std::collections::BTreeMap;

use crate::ui::desktop_protocol::endpoint::HostEndpoint;
use crate::ui::desktop_protocol::host::SurfaceStore;
use crate::ui::desktop_protocol::message::DrawList;
use crate::ui::desktop_protocol::shm::SharedFrameBuffer;
use crate::ui::desktop_protocol::transport::Transport;
use crate::ui::session::{AppId, Wid};

/// 一条孵化连接的宿主侧状态（多 client 宿主的"每 App 一份"部分）。
pub struct BrokerClient {
    /// per-app 管道名（map 键冗余存一份，便于日志/回收）。
    pub pipe: String,
    pub end: Box<dyn Transport + Send>,
    pub endpoint: HostEndpoint,
    pub(crate) surfaces: SurfaceStore,
    /// surface → shm 段（FrameReadyShared 载荷源）。
    pub(crate) shm: BTreeMap<u64, SharedFrameBuffer>,
    /// wid → surface 句柄。
    pub(crate) wid_surface: BTreeMap<u64, u64>,
    /// 落地后的会话对象（Active 后有效）。
    pub app_id: Option<AppId>,
    pub wid: Option<Wid>,
    /// 孵化上报的 app 名（ResolveAndAttach 落地时回填；压测按名寻窗）。
    pub app_name: Option<String>,
}

impl BrokerClient {
    pub fn new(pipe: String, end: Box<dyn Transport + Send>) -> Self {
        Self {
            pipe,
            end,
            endpoint: HostEndpoint::listen(),
            surfaces: SurfaceStore::new(),
            shm: BTreeMap::new(),
            wid_surface: BTreeMap::new(),
            app_id: None,
            wid: None,
            app_name: None,
        }
    }

    /// 该 client 的虚拟窗当前合成面（压测帧断言口）。
    pub fn composed(&self) -> Option<&DrawList> {
        let surface = *self.wid_surface.get(&self.wid?.0)?;
        self.surfaces.front(surface)
    }
}

// ---------------------------------------------------------------------------
// S5：child 进程内存采样（Windows `K32GetProcessMemoryInfo` FFI，零新
// 依赖）。度量口径 = **边际增量**（N=1→3→5 每增一个 child 的均摊增量，
// 见计划待澄清①）；WorkingSet / PrivateUsage 双字段。
// ---------------------------------------------------------------------------

/// 一次进程内存采样（字节）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessMemorySample {
    /// WorkingSet（物理驻留）。
    pub working_set: u64,
    /// PrivateUsage（提交私有字节；比 WorkingSet 更贴近"App 净增成本"）。
    pub private_bytes: u64,
}

#[cfg(windows)]
mod mem_ffi {
    // kernel32 导出（Win7+；避免引入 psapi.lib 链接面）。
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> isize;
        fn CloseHandle(hObject: isize) -> i32;
        fn K32GetProcessMemoryInfo(
            hProcess: isize,
            ppsmemCounters: *mut PROCESS_MEMORY_COUNTERS,
            cb: u32,
        ) -> i32;
    }

    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

    /// PROCESS_MEMORY_COUNTERS + EX 尾字段 PrivateUsage（cb 按全结构
    /// 传入，PSAPI 会把 PrivateUsage 一并填充）。
    #[repr(C)]
    pub struct PROCESS_MEMORY_COUNTERS {
        pub cb: u32,
        pub PageFaultCount: u32,
        pub PeakWorkingSetSize: usize,
        pub WorkingSetSize: usize,
        pub QuotaPeakPagedPoolUsage: usize,
        pub QuotaPagedPoolUsage: usize,
        pub QuotaPeakNonPagedPoolUsage: usize,
        pub QuotaNonPagedPoolUsage: usize,
        pub PagefileUsage: usize,
        pub PeakPagefileUsage: usize,
        pub PrivateUsage: usize,
    }

    pub fn query(pid: u32) -> Result<super::ProcessMemorySample, String> {
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle == 0 {
                return Err(format!("OpenProcess({pid}): {}", std::io::Error::last_os_error()));
            }
            let mut counters = PROCESS_MEMORY_COUNTERS {
                cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
                PageFaultCount: 0,
                PeakWorkingSetSize: 0,
                WorkingSetSize: 0,
                QuotaPeakPagedPoolUsage: 0,
                QuotaPagedPoolUsage: 0,
                QuotaPeakNonPagedPoolUsage: 0,
                QuotaNonPagedPoolUsage: 0,
                PagefileUsage: 0,
                PeakPagefileUsage: 0,
                PrivateUsage: 0,
            };
            let ok = K32GetProcessMemoryInfo(
                handle,
                &mut counters,
                counters.cb,
            );
            let _ = CloseHandle(handle);
            if ok == 0 {
                return Err(format!(
                    "K32GetProcessMemoryInfo({pid}): {}",
                    std::io::Error::last_os_error()
                ));
            }
            Ok(super::ProcessMemorySample {
                working_set: counters.WorkingSetSize as u64,
                private_bytes: counters.PrivateUsage as u64,
            })
        }
    }
}

/// 采样一个进程的内存（字节）。非 Windows 平台返回 Err（v1 压测宿主
/// = Windows 桌面；Linux memfd 宿主见 shm.rs 注记）。
pub fn sample_process_memory(pid: u32) -> Result<ProcessMemorySample, String> {
    #[cfg(windows)]
    {
        mem_ffi::query(pid)
    }
    #[cfg(not(windows))]
    {
        let _ = pid;
        Err("memory sampling is windows-only in v1".into())
    }
}

// ---------------------------------------------------------------------------
// 压测 harness：N child → broker 孵化 → 全 Active → 逐 App 点击帧递增 →
// 30s 稳定存活（子进程 = 测试二进制 re-exec 走真协议主循环）。
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::ui::desktop_protocol::broker;
    use crate::ui::desktop_protocol::client_runtime::{
        run_client, AppProjector, ClientConfig, ReconnectPolicy,
    };
    use crate::ui::desktop_protocol::message::{DrawOp, MouseButton};
    use crate::ui::desktop_protocol::transport;
    use crate::ui::session::{DesktopSession, LaunchSpec};

    /// 子进程识别 env：broker 管道名 + app 名。
    const CHILD_BROKER_ENV: &str = "AUTO_480_BROKER";
    const CHILD_APP_ENV: &str = "AUTO_480_APP";
    const CHILD_MARKER: &str = "AUTO480-CHILD";

    /// 压测 App 源（每个 child 一份同源计数器；widget 名无关进程边界）。
    const STRESS_SRC: &str = "widget StressCounter {\n    model { var count int = 0 }\n    view {\n        button \"+\" { onclick: () => {.count += 1} }\n        text `count: ${.count}`\n    }\n}\n";

    /// 子进程体：request_incubation → ClientPump::run 真协议主循环，直至
    /// 宿主 Close（压测收尾）或重连预算耗尽。直接跑套件（无 env）时跳过。
    #[test]
    fn stage3_child_body() {
        let Ok(broker_pipe) = std::env::var(CHILD_BROKER_ENV) else {
            return;
        };
        let app_name = std::env::var(CHILD_APP_ENV).expect("app name env");

        let (per_app_pipe, end) =
            broker::request_incubation(&broker_pipe, &app_name, 10_000).expect("incubate");
        let component = crate::build_dynamic_component(STRESS_SRC, None).expect("child build");
        let config = ClientConfig {
            app_name: app_name.clone(),
            title: app_name,
            width: 480.0,
            height: 320.0,
        };
        let reconnect =
            ReconnectPolicy { pipe: per_app_pipe, budget_ms: 30_000, interval_ms: 50 };
        let projector = AppProjector::new(component, 480.0, 320.0);
        let (exit, proj) = run_client(end, projector, config, Some(reconnect));
        println!("{CHILD_MARKER} exit={exit:?} rev={}", proj.revision());
    }

    /// 压测窗口布局：落地后把每个虚拟窗搬开（非重叠网格），点击可定向。
    fn place_window(session: &mut DesktopSession, wid: Wid, index: usize) {
        if let Some(host) = session.host.as_mut() {
            if let Some(v) = host.wm.wins.get_mut(&wid) {
                let mut rect = *v.rect.borrow();
                rect.x = 16.0 + 500.0 * index as f32;
                rect.y = 16.0;
                *v.rect.borrow_mut() = rect;
            }
        }
    }

    /// re-exec 子进程（剥离 NEXTEST_* 守护）。
    fn spawn_children(broker_pipe: &str, names: &[String]) -> Vec<std::process::Child> {
        let exe = std::env::current_exe().expect("current_exe");
        let mut children = Vec::new();
        for app in names {
            let mut cmd = std::process::Command::new(&exe);
            cmd.args(["stage3_child_body", "--test-threads", "1", "--nocapture"])
                .env(CHILD_BROKER_ENV, broker_pipe)
                .env(CHILD_APP_ENV, app)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::inherit());
            for (k, _) in std::env::vars() {
                if k.starts_with("NEXTEST_") {
                    cmd.env_remove(&k);
                }
            }
            children.push(cmd.spawn().expect("spawn child"));
        }
        children
    }

    /// 孵化 attach 至 `want` 个落地（30s 预算）。
    fn attach_until(session: &mut DesktopSession, want: usize) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while session.broker_clients.values().filter_map(|c| c.wid).count() < want
            && std::time::Instant::now() < deadline
        {
            session.attach_pending_incubations(2000);
            session.pump_broker_clients();
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    /// child 总内存采样（逐 pid WorkingSet/PrivateUsage 求和）。
    fn sample_children_total(children: &[std::process::Child]) -> Option<ProcessMemorySample> {
        let mut total = ProcessMemorySample { working_set: 0, private_bytes: 0 };
        for child in children {
            let s = sample_process_memory(child.id()).ok()?;
            total.working_set += s.working_set;
            total.private_bytes += s.private_bytes;
        }
        Some(total)
    }

    /// N App 压测主体：spawn → 孵化 attach → 全 Active → 逐 App 点击帧
    /// 递增 → `stability_secs` 稳定存活（期间持续泵帧）→ 收尾 Close。
    fn stress_body(n: usize, stability_secs: u64) {
        use std::sync::atomic::AtomicBool;

        let broker_pipe = format!("autodesk-broker-s4-{}-n{n}", std::process::id());

        // ---- 桌面侧：真实会话 + 注册表（n 个计数器 App）+ enable_broker。
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        let names: Vec<String> = (0..n).map(|i| format!("stress-{i}")).collect();
        let src = STRESS_SRC.to_string();
        let known = names.clone();
        session.desktop.app_resolver =
            Some(std::sync::Arc::new(move |name: &str| {
                if known.iter().any(|n| n == name) {
                    Some(LaunchSpec {
                        code: src.clone(),
                        source_path: None,
                        title: Some(name.to_string()),
                    })
                } else {
                    None
                }
            }));
        let stop = Arc::new(AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));

        // ---- spawn N 个 child（re-exec；剥离 NEXTEST_* 守护）。
        let mut children = spawn_children(&broker_pipe, &names);

        // ---- 孵化 attach 循环：直到 N 个全落地（30s 预算）。
        attach_until(&mut session, n);
        let landed: Vec<Wid> = session.broker_clients.values().filter_map(|c| c.wid).collect();
        assert_eq!(landed.len(), n, "全部 child 孵化落地（30s 预算）");
        assert_eq!(session.apps.len(), n, "n 条 AppSession");
        for (i, app) in names.iter().enumerate() {
            let wid = session
                .broker_clients
                .values()
                .find(|c| c.app_name.as_deref() == Some(app.as_str()))
                .and_then(|c| c.wid)
                .expect("landed");
            place_window(&mut session, wid, i);
        }
        let wids: Vec<Wid> = names
            .iter()
            .map(|app| {
                session
                    .broker_clients
                    .values()
                    .find(|c| c.app_name.as_deref() == Some(app.as_str()))
                    .and_then(|c| c.wid)
                    .expect("landed")
            })
            .collect();

        // ---- 逐 App 点击：帧 count 递增（shm 载荷断言）。
        for (i, wid) in wids.iter().enumerate() {
            let base_x = 16.0 + 500.0 * i as f32;
            let routed = session.broker_pointer_down(base_x + 60.0, 16.0 + 40.0, MouseButton::Left);
            assert!(routed, "App {i} 点击路由");
            let mut seen = false;
            for _ in 0..300 {
                session.pump_broker_clients();
                let app = &names[i];
                let hit = session
                    .broker_clients
                    .values()
                    .find(|c| c.app_name.as_deref() == Some(app.as_str()))
                    .and_then(|c| c.composed())
                    .is_some_and(|list| {
                        list.ops.iter().any(|op| matches!(op,
                            DrawOp::Text { text, .. } if text == "count: 1"))
                    });
                if hit {
                    seen = true;
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            assert!(seen, "App {i} 帧递增到 count: 1");
        }

        // ---- 稳定存活：持续泵 `stability_secs`，全程全 child 存活、全
        // client 在册。
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(stability_secs);
        while std::time::Instant::now() < deadline {
            session.pump_broker_clients();
            for child in children.iter_mut() {
                assert!(child.try_wait().expect("try_wait").is_none(), "child 提前退出");
            }
            assert_eq!(session.broker_clients.len(), n, "client 全在册");
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        // ---- S5 采样集成：稳定窗末采集 N=child 全体内存（数字供 S6 报告）。
        if let Some(total) = sample_children_total(&children) {
            println!(
                "AUTO480-MEM stage=N{n} working_set={}B private={}B",
                total.working_set, total.private_bytes
            );
        }

        // ---- 收尾：逐 client Close → ExitRequest → 回收；child 退出码 0。
        let closes: Vec<(String, Option<crate::ui::desktop_protocol::message::ProtocolMsg>)> =
            session
                .broker_clients
                .values_mut()
                .map(|c| (c.pipe.clone(), c.endpoint.close().ok()))
                .collect();
        for (pipe, close) in closes {
            if let Some(close) = close {
                if let Some(c) = session.broker_clients.get_mut(&pipe) {
                    let _ = c.end.send(&close);
                }
            }
        }
        for _ in 0..300 {
            session.pump_broker_clients();
            if session.apps.is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        for child in children.iter_mut() {
            let start = std::time::Instant::now();
            let status = loop {
                if let Some(s) = child.try_wait().expect("try_wait") {
                    break s;
                }
                assert!(start.elapsed() < std::time::Duration::from_secs(30), "child 收尾超时");
                std::thread::sleep(std::time::Duration::from_millis(50));
            };
            assert!(status.success(), "child 退出码 {status}");
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = transport::connect(&broker_pipe, 500);
    }

    /// N=3 压测（先绿档）。
    #[test]
    fn stage3_multi_app_stress_n3() {
        stress_body(3, 30);
    }

    /// N=5 压测（stress 档）。
    #[test]
    fn stage3_multi_app_stress_n5() {
        stress_body(5, 30);
    }

    /// S7 弹性重连：host 断连（server drop → child EOF）→ child 存活
    /// 等待重连（不退出、状态保持）→ 宿主重建管道 → child 连回、握手
    /// 续跑 → 再点击 count 连续（2）、revision 连续（3）。
    #[test]
    fn stage3_reconnect_state_continuous() {
        use crate::ui::desktop_protocol::client_runtime::ClientPump;
        use crate::ui::desktop_protocol::host::ProtocolHost;
        use crate::ui::desktop_protocol::message::DrawOp;
        use crate::ui::session::DesktopSession;

        const SRC: &str = "widget ReconnectCounter {
    model { var count int = 0 }
    view {
        button \"+\" { onclick: () => {.count += 1} }
        text `count: ${.count}`
    }
}
";

        let pipe = format!("autodesk-reconnect-s7-{}", std::process::id());
        let config = ClientConfig {
            app_name: "reconnect-counter".into(),
            title: "reconnect".into(),
            width: 480.0,
            height: 320.0,
        };

        // ---- 第一条连接：listen → child 泵（重连策略 10s 预算）。
        let listener = transport::listen(&pipe).expect("listen A");
        let app_end = transport::connect(&pipe, 2000).expect("connect A");
        let mut client = ClientPump::new(
            app_end,
            {
                let component = crate::build_dynamic_component(SRC, None).expect("build");
                AppProjector::new(component, 480.0, 320.0)
            },
            config.clone(),
            Some(ReconnectPolicy { pipe: pipe.clone(), budget_ms: 10_000, interval_ms: 20 }),
        );
        let mut server_end = listener.wait_connect().expect("server A");

        let mut session = DesktopSession::__test_session();
        session.__test_open_desktop();
        let src = SRC;
        let mut ph = ProtocolHost::new(&mut session, move |name: &str| {
            if name == "reconnect-counter" {
                crate::build_dynamic_component(src, None).map_err(|e| format!("{e}"))
            } else {
                Err(format!("unknown app {name}"))
            }
        });
        fn pump(server_end: &mut Box<dyn crate::ui::desktop_protocol::transport::Transport + Send>, ph: &mut ProtocolHost<'_>) {
            while let Some(loaded) = server_end.try_recv() {
                let msg = loaded.expect("解码");
                ph.handle(&msg).expect("host 状态机");
                for reply in std::mem::take(&mut ph.to_app) {
                    let _ = server_end.send(&reply);
                }
            }
        }

        // 泵到 Active + 首帧，点击一次（count=1）。
        let mut wid = None;
        for _ in 0..200 {
            pump(&mut server_end, &mut ph);
            let _ = client.step();
            if !ph.session.apps.is_empty() {
                wid = ph.active().1;
                if ph.composed(wid.expect("wid").0).is_some() {
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let wid_a = wid.expect("连接 A 孵化");
        let injected = ph.pointer_down(60.0, 40.0, MouseButton::Left).expect("窗内命中");
        server_end.send(&injected).unwrap();
        let mut count_a = 0;
        for _ in 0..200 {
            pump(&mut server_end, &mut ph);
            let _ = client.step();
            if ph.composed(wid_a.0).is_some_and(|l| l.ops.iter().any(|op| matches!(op,
                DrawOp::Text { text, .. } if text == "count: 1"))) {
                count_a = 1;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(count_a, 1, "断连前点击生效");

        // ---- host 断连：server 端 drop → child EOF → 存活等待重连。
        server_end.send(&ph.endpoint.close().unwrap_or(
            crate::ui::desktop_protocol::message::ProtocolMsg::Handshake(
                crate::ui::desktop_protocol::message::HandshakeMsg::Ready))); // no-op 填充，真实断连靠 drop
        drop(ph);
        drop(server_end); // EOF 传播给 child
        for _ in 0..50 {
            assert!(client.step().is_none(), "child 在断连期存活（不退出）");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // ---- 宿主重建管道：同名单独 listen；旧 AppSession/窗按 EOF 回收语义清场。
        session.apps.clear();
        if let Some(host) = session.host.as_mut() {
            host.wm.wins.clear();
        }
        let listener_b = transport::listen(&pipe).expect("listen B");
        // 驱动 child 重连：connect 只在 step() 内尝试（try_reconnect），
        // 故 accept 放侧线程，主循环泵 step 直至 child 连回。
        let accept = std::thread::spawn(move || listener_b.wait_connect().expect("server B"));
        for _ in 0..200 {
            let _ = client.step();
            if accept.is_finished() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let mut server_end = accept.join().expect("accept thread");

        // 新连接（B）：新 ProtocolHost 端点（新 wid/surface），VM 状态在 child 侧。
        let mut ph = ProtocolHost::new(&mut session, move |name: &str| {
            if name == "reconnect-counter" {
                crate::build_dynamic_component(src, None).map_err(|e| format!("{e}"))
            } else {
                Err(format!("unknown app {name}"))
            }
        });
        let mut wid_b = None;
        for _ in 0..300 {
            pump(&mut server_end, &mut ph);
            let _ = client.step();
            if !ph.session.apps.is_empty() {
                wid_b = ph.active().1;
                if ph.composed(wid_b.expect("wid").0).is_some() {
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let wid_b = wid_b.expect("重连后再次孵化");

        // ---- 再点击：count 连续（2 = 断连前 1 + 现 successes），revision 连续。
        let injected = ph.pointer_down(60.0, 40.0, MouseButton::Left).expect("窗内命中");
        server_end.send(&injected).unwrap();
        let mut count_b = 0;
        for _ in 0..300 {
            pump(&mut server_end, &mut ph);
            let _ = client.step();
            if ph.composed(wid_b.0).is_some_and(|l| l.ops.iter().any(|op| matches!(op,
                DrawOp::Text { text, .. } if text == "count: 2"))) {
                count_b = 2;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(count_b, 2, "重连后 count 连续（断连前 1 + 断连后 1）");

        // 出口核对：L2Detach 走完，projector 状态/revision 连续（rev=1+2）。
        let detach = ph.endpoint.l2_detach().unwrap();
        server_end.send(&detach).unwrap();
        let (exit, projector) = loop {
            pump(&mut server_end, &mut ph);
            if let Some(done) = client.step() {
                break done;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        };
        assert_eq!(exit, crate::ui::desktop_protocol::client_runtime::ClientExit::L2Detached);
        assert_eq!(projector.read_state("count").unwrap(), auto_val::Value::Int(2), "count 连续");
        assert_eq!(projector.revision(), 3, "revision 连续（1 + 2 次点击）");
    }

    /// S5 采样单测 + N=1/3/5 边际增量数字生成：阶段化 spawn（1 → 3 →
    /// 5 child），每阶段 settle 后采全体 child WorkingSet/PrivateUsage；
    /// 断言数值 >0 且 N=5 > N=1；边际增量打印供 S6 报告引用。
    #[test]
    fn stage3_memory_baseline_n1_3_5() {
        use std::sync::atomic::AtomicBool;

        let broker_pipe = format!("autodesk-broker-s4-mem-{}", std::process::id());
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        let names: Vec<String> = (0..5).map(|i| format!("mem-{i}")).collect();
        let src = STRESS_SRC.to_string();
        let known = names.clone();
        session.desktop.app_resolver =
            Some(std::sync::Arc::new(move |name: &str| {
                if known.iter().any(|n| n == name) {
                    Some(LaunchSpec {
                        code: src.clone(),
                        source_path: None,
                        title: Some(name.to_string()),
                    })
                } else {
                    None
                }
            }));
        let stop = Arc::new(AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));

        // 阶段化 spawn+attach：1 → 3 → 5。attach 是 drain-all 语义，故
        // 每批先 spawn 再 attach；批次间 settle 1.5s 后采样（早批次多出
        // 的 settle 时间使边际估计偏保守——对 1-5MB/App 判定方向安全）。
        let mut children: Vec<std::process::Child> = Vec::new();
        let mut samples = Vec::new();
        for (stage, want) in [1usize, 3, 5].into_iter().enumerate() {
            let batch = &names[children.len()..want];
            children.extend(spawn_children(&broker_pipe, batch));
            attach_until(&mut session, want);
            let landed = session.broker_clients.values().filter_map(|c| c.wid).count();
            assert_eq!(landed, want, "阶段 N={want} 全落地");
            session.pump_broker_clients();
            std::thread::sleep(std::time::Duration::from_millis(1500));
            let total = sample_children_total(&children).expect("windows 采样");
            samples.push(total);
            println!(
                "AUTO480-MEM stage=N{want} children={want} working_set={}B ({:.1}MiB) private={}B ({:.1}MiB)",
                total.working_set,
                total.working_set as f64 / 1048576.0,
                total.private_bytes,
                total.private_bytes as f64 / 1048576.0,
            );
        }

        // 判定：数值 >0；N=5 > N=1（边际增量为正）。
        for (stage, s) in samples.iter().enumerate() {
            assert!(s.working_set > 0, "N={} WorkingSet > 0", [1, 3, 5][stage]);
            assert!(s.private_bytes > 0, "N={} PrivateUsage > 0", [1, 3, 5][stage]);
        }
        assert!(samples[2].working_set > samples[0].working_set, "N=5 WorkingSet > N=1");
        assert!(samples[2].private_bytes > samples[0].private_bytes, "N=5 PrivateUsage > N=1");

        // 边际增量（S6 报告口径）：(N=5 − N=1) / 4。
        let ws_marginal = (samples[2].working_set - samples[0].working_set) / 4;
        let priv_marginal = (samples[2].private_bytes - samples[0].private_bytes) / 4;
        println!(
            "AUTO480-MEM marginal-per-app working_set={ws_marginal}B ({:.2}MiB) private={priv_marginal}B ({:.2}MiB)",
            ws_marginal as f64 / 1048576.0,
            priv_marginal as f64 / 1048576.0,
        );

        // 收尾：Close 全部 → child 退出。
        let closes: Vec<(String, Option<crate::ui::desktop_protocol::message::ProtocolMsg>)> =
            session
                .broker_clients
                .values_mut()
                .map(|c| (c.pipe.clone(), c.endpoint.close().ok()))
                .collect();
        for (pipe, close) in closes {
            if let Some(close) = close {
                if let Some(c) = session.broker_clients.get_mut(&pipe) {
                    let _ = c.end.send(&close);
                }
            }
        }
        for _ in 0..300 {
            session.pump_broker_clients();
            if session.apps.is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let mut children = children;
        for child in children.iter_mut() {
            let start = std::time::Instant::now();
            let status = loop {
                if let Some(st) = child.try_wait().expect("try_wait") {
                    break st;
                }
                assert!(start.elapsed() < std::time::Duration::from_secs(30), "child 收尾超时");
                std::thread::sleep(std::time::Duration::from_millis(50));
            };
            assert!(status.success(), "child 退出码 {status}");
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = transport::connect(&broker_pipe, 500);
    }
}

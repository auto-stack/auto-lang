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
use crate::ui::desktop_protocol::message::{DrawList, FrameMsg};
use crate::ui::desktop_protocol::shm::SharedFrameBuffer;
use crate::ui::desktop_protocol::transport::Transport;
use crate::ui::session::{AppId, Wid};

/// 像素臂前缓冲（v1.3）：一条 surface 的最新 RGBA 帧（宿主渲染臂据此
/// 上传纹理；宿主侧"每 App 一份"表面驻留，与 SurfaceStore 命令帧同型）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelsSurface {
    pub rgba: Vec<u8>,
    pub w: u32,
    pub h: u32,
    pub stride: u32,
    pub revision: u64,
}

/// 一条孵化连接的宿主侧状态（多 client 宿主的"每 App 一份"部分）。
pub struct BrokerClient {
    /// per-app 管道名（map 键冗余存一份，便于日志/回收）。
    pub pipe: String,
    pub end: Box<dyn Transport + Send>,
    pub endpoint: HostEndpoint,
    pub(crate) surfaces: SurfaceStore,
    /// surface → shm 段（FrameReadyShared/FrameReadyPixels 载荷源）。
    pub(crate) shm: BTreeMap<u64, SharedFrameBuffer>,
    /// wid → surface 句柄。
    pub(crate) wid_surface: BTreeMap<u64, u64>,
    /// surface → 像素前缓冲（v1.3 independent 臂；Commands 臂不用）。
    pub(crate) pixels: BTreeMap<u64, PixelsSurface>,
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
            pixels: BTreeMap::new(),
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

    /// 该 client 的像素前缓冲（v1.3 渲染臂/测试断言口）。
    pub fn composed_pixels(&self) -> Option<&PixelsSurface> {
        let surface = *self.wid_surface.get(&self.wid?.0)?;
        self.pixels.get(&surface)
    }

    /// 像素帧合成（v1.3）：shm 槽读 RGBA → 前缓冲翻面。返回 FrameAck
    /// （frame_id 原样回带；None = 无段/读失败，调用方不回 ack）。
    pub fn compose_pixels(
        &mut self,
        surface: u64,
        wid: u64,
        frame_id: u64,
        slot: u8,
        revision: u64,
        w: u32,
        h: u32,
        stride: u32,
    ) -> Option<FrameMsg> {
        let rgba = self.shm.get(&surface)?.read_slot(slot).ok()?;
        self.pixels.insert(
            surface,
            PixelsSurface { rgba, w, h, stride, revision },
        );
        Some(FrameMsg::FrameAck { wid, frame_id, slot })
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

/// L3 v2a：融合态 App 的 AutoVM 状态快照编码（revision + 全部原始状态
/// 字段；复合类型落 Nil 占位，见 client_runtime::encode_state_snapshot）。
pub fn fused_state_snapshot(
    component: &crate::ui::dynamic::DynamicComponent,
    revision: u64,
) -> Vec<u8> {
    let fields: Vec<(String, auto_val::Value)> =
        component.read_all_state().into_iter().collect();
    super::client_runtime::encode_state_snapshot(revision, &fields)
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
    use crate::ui::desktop_protocol::message::{DrawOp, FrameMode, MouseButton};
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

    // -----------------------------------------------------------------------
    // Plan 500 步骤 8 —— T3 re-exec 集成：001–005 queue 端到端 +
    // independent 像素帧 + 双模并存。
    // -----------------------------------------------------------------------

    /// 子进程识别 env（模式档：queue | independent）。
    const T3_BROKER_ENV: &str = "AUTO_500_BROKER";
    const T3_APP_ENV: &str = "AUTO_500_APP";
    const T3_MODE_ENV: &str = "AUTO_500_MODE";

    /// T3 表面尺寸（005-login 内容高 ~812px，320 高度会溢出表面）。
    const T3_W: f32 = 480.0;
    const T3_H: f32 = 900.0;

    /// 001–005 示例名（与 examples/ui 目录一致）。
    const T3_EXAMPLES: [&str; 5] = [
        "001-helloworld",
        "002-counter",
        "003-converter",
        "004-profile-card",
        "005-login",
    ];

    fn example_source(dir: &str) -> String {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/ui/P/src/front/app.at"
        )
        .replace('P', dir);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
    }

    /// T3 子进程体：三态裁决的 child 侧（queue = 真协议泵；independent =
    /// 真 iced 隐藏窗 + 截图泵）。直接跑套件（无 env）时跳过。
    #[test]
    fn t3_child_body() {
        let Ok(broker_pipe) = std::env::var(T3_BROKER_ENV) else {
            return;
        };
        let app = std::env::var(T3_APP_ENV).expect("app env");
        let mode = std::env::var(T3_MODE_ENV).unwrap_or_else(|_| "queue".into());
        let src = example_source(&app);
        let component = crate::build_dynamic_component(&src, None).expect("child build");
        match mode.as_str() {
            "independent" => {
                let render = broker::RequestedRender {
                    mode: crate::ui::desktop_protocol::message::FrameMode::Pixels,
                    auto_downgraded: false,
                };
                let (_pipe, end) =
                    broker::request_incubation_render(&broker_pipe, &app, render, 10_000)
                        .expect("incubate");
                crate::ui::desktop_protocol::pixels::run_independent_child(
                    end, component, &app, &app, T3_W, T3_H,
                )
                .expect("independent child");
            }
            _ => {
                let (_pipe, end) = broker::request_incubation_render(
                    &broker_pipe,
                    &app,
                    broker::RequestedRender::default(),
                    10_000,
                )
                .expect("incubate");
                let config =
                    ClientConfig { app_name: app.clone(), title: app, width: T3_W, height: T3_H };
                let reconnect =
                    ReconnectPolicy { pipe: _pipe, budget_ms: 30_000, interval_ms: 50 };
                let projector = AppProjector::new(component, T3_W, T3_H);
                let (exit, proj) = run_client(end, projector, config, Some(reconnect));
                println!("AUTO500-CHILD exit={exit:?} rev={}", proj.revision());
            }
        }
    }

    /// re-exec 一个 T3 子进程（env 注入 broker 管道/app 名/模式档）。
    fn spawn_t3_child(broker_pipe: &str, app: &str, mode: &str) -> std::process::Child {
        let exe = std::env::current_exe().expect("current_exe");
        let mut cmd = std::process::Command::new(&exe);
        cmd.args(["t3_child_body", "--test-threads", "1", "--nocapture"])
            .env(T3_BROKER_ENV, broker_pipe)
            .env(T3_APP_ENV, app)
            .env(T3_MODE_ENV, mode)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit());
        for (k, _) in std::env::vars() {
            if k.starts_with("NEXTEST_") {
                cmd.env_remove(&k);
            }
        }
        cmd.spawn().expect("spawn t3 child")
    }

    /// 按示例名取该 client 的合成帧（queue 臂 DrawList）。
    fn composed_texts(session: &DesktopSession, app: &str) -> Vec<String> {
        session
            .broker_clients
            .values()
            .find(|c| c.app_name.as_deref() == Some(app))
            .and_then(|c| c.composed())
            .map(|list| {
                list.ops
                    .iter()
                    .filter_map(|op| match op {
                        crate::ui::desktop_protocol::message::DrawOp::Text { text, .. } => {
                            Some(text.clone())
                        }
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 等待条件成立（泵帧 + 谓词轮询）。
    fn wait_frames(
        session: &mut DesktopSession,
        mut pred: impl FnMut(&DesktopSession) -> bool,
    ) -> bool {
        for _ in 0..600 {
            session.pump_broker_clients();
            if pred(session) {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        false
    }

    /// T3 主体一：001–005 五示例 queue 模式端到端（re-exec 真子进程 +
    /// 真协议泵）——孵化全 Active → 逐示例交互闭环（本地孪生投影器出
    /// 命中坐标，与 child 同引擎同源确定性布局）→ 帧内容断言 → 收尾。
    #[test]
    fn t3_examples_queue_end_to_end() {
        let broker_pipe = format!("autodesk-broker-t3-{}", std::process::id());
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        let names: Vec<String> = T3_EXAMPLES.iter().map(|s| s.to_string()).collect();
        let sources: Vec<(String, String)> = names
            .iter()
            .map(|n| (n.clone(), example_source(n)))
            .collect();
        session.desktop.app_resolver = Some(std::sync::Arc::new(move |name: &str| {
            sources
                .iter()
                .find(|(n, _)| n == name)
                .map(|(n, src)| LaunchSpec {
                    code: src.clone(),
                    source_path: None,
                    title: Some(n.clone()),
                    name: None,
                    fit: false,
                    daemon: None,
                    back_root: None,
                })
        }));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));

        // 孪生投影器（同源布局 → 命中坐标）。
        let twins: Vec<(String, crate::ui::desktop_protocol::client_runtime::AppProjector)> =
            names
                .iter()
                .map(|n| {
                    let src = example_source(n);
                    let comp = crate::build_dynamic_component(&src, None).expect("twin build");
                    let mut p = crate::ui::desktop_protocol::client_runtime::AppProjector::new(
                        comp, T3_W, T3_H,
                    );
                    {
                        use crate::ui::desktop_protocol::endpoint::FrameSource;
                        p.render_frame();
                    }
                    (n.clone(), p)
                })
                .collect();
        let twin_hit = |app: &str, needle: &str| -> Option<(f32, f32)> {
            twins
                .iter()
                .find(|(n, _)| n == app)
                .and_then(|(_, p)| {
                    p.hit_regions()
                        .into_iter()
                        .find(|(_, k)| k.contains(needle))
                        .map(|(r, _)| (r.x + r.w / 2.0, r.y + r.h / 2.0))
                })
        };
        /// 命中区序选（row 内第 idx 个匹配；002 的 "+" 是第三个按钮）。
        let twin_hit_nth = |app: &str, needle: &str, idx: usize| -> Option<(f32, f32)> {
            twins
                .iter()
                .find(|(n, _)| n == app)
                .and_then(|(_, p)| {
                    p.hit_regions()
                        .into_iter()
                        .filter(|(_, k)| k.contains(needle))
                        .nth(idx)
                        .map(|(r, _)| (r.x + r.w / 2.0, r.y + r.h / 2.0))
                })
        };

        let mut children: Vec<std::process::Child> =
            names.iter().map(|n| spawn_t3_child(&broker_pipe, n, "queue")).collect();

        attach_until(&mut session, names.len());
        let landed = session.broker_clients.values().filter_map(|c| c.wid).count();
        assert_eq!(landed, names.len(), "五示例全孵化落地");
        let wids: Vec<Wid> =
            session.broker_clients.values().filter_map(|c| c.wid).collect();
        for (i, wid) in wids.into_iter().enumerate() {
            place_window(&mut session, wid, i);
        }

        // ---- 逐示例交互闭环（坐标 = 孪生命中区中心 + 窗原点）。 ----
        let origin_of = |session: &DesktopSession, app: &str| -> Option<(f32, f32)> {
            session.broker_clients.values().find(|c| c.app_name.as_deref() == Some(app))
                .and_then(|c| c.wid)
                .and_then(|wid| {
                    session.host.as_ref().and_then(|h| h.wm.wins.get(&wid))
                        .map(|v| { let r = *v.rect.borrow(); (r.x, r.y) })
                })
        };

        // 001/004：帧到位即断言（无交互 widget）。
        assert!(
            wait_frames(&mut session, |s| composed_texts(s, "001-helloworld")
                .iter()
                .any(|t| t == "Hello, World!")),
            "001 帧文本到位"
        );
        assert!(
            wait_frames(&mut session, |s| composed_texts(s, "004-profile-card")
                .iter()
                .any(|t| t == "Jane Cooper")),
            "004 帧文本到位"
        );

        // 002：点击 "+" → Counter: 1。
        {
            // 002 行序：- Reset + → "+" 是第 3 个按钮命中区。
            let (hx, hy) = twin_hit_nth("002-counter", "button:", 2)
                .expect("002 孪生按钮坐标（+ 为第三个）");
            let (ox, oy) = origin_of(&session, "002-counter").expect("002 窗原点");
            assert!(
                session.broker_pointer_down(ox + hx, oy + hy, MouseButton::Left),
                "002 点击路由"
            );
            assert!(
                wait_frames(&mut session, |s| composed_texts(s, "002-counter")
                    .iter()
                    .any(|t| t == "Counter: 1")),
                "002 点击后帧递增"
            );
        }

        // 003：聚焦 celsius 输入 → 输入 100 → 联动 212。
        {
            let (hx, hy) = twin_hit("003-converter", "input:celsius").expect("003 celsius 坐标");
            let (ox, oy) = origin_of(&session, "003-converter").expect("003 窗原点");
            assert!(session.broker_pointer_down(ox + hx, oy + hy, MouseButton::Left));
            let (wid, mut end_pipe) = {
                let client = session
                    .broker_clients
                    .values_mut()
                    .find(|c| c.app_name.as_deref() == Some("003-converter"))
                    .expect("003 client");
                (client.wid.expect("wid"), client.pipe.clone())
            };
            let _ = &mut end_pipe;
            for ch in ['1', '0', '0'] {
                let msg = crate::ui::desktop_protocol::message::ProtocolMsg::Input(
                    crate::ui::desktop_protocol::message::InputMsg::CharTyped {
                        wid: wid.0,
                        ch,
                    },
                );
                if let Some(client) = session
                    .broker_clients
                    .values_mut()
                    .find(|c| c.app_name.as_deref() == Some("003-converter"))
                {
                    let _ = client.end.send(&msg);
                }
            }
            assert!(
                wait_frames(&mut session, |s| {
                    let t = composed_texts(s, "003-converter");
                    t.iter().any(|x| x == "212") && t.iter().any(|x| x == "100")
                }),
                "003 输入换算联动: {:?}",
                composed_texts(&session, "003-converter")
            );
        }

        // 005：聚焦 email 输入 → 输入 a@b.c → 点 Sign In → password 错误显示。
        {
            let (hx, hy) = twin_hit("005-login", "input:email").expect("005 email 坐标");
            let (ox, oy) = origin_of(&session, "005-login").expect("005 窗原点");
            assert!(session.broker_pointer_down(ox + hx, oy + hy, MouseButton::Left));
            let wid = session
                .broker_clients
                .values()
                .find(|c| c.app_name.as_deref() == Some("005-login"))
                .and_then(|c| c.wid)
                .expect("005 wid");
            for ch in "a@b.c".chars() {
                let msg = crate::ui::desktop_protocol::message::ProtocolMsg::Input(
                    crate::ui::desktop_protocol::message::InputMsg::CharTyped {
                        wid: wid.0,
                        ch,
                    },
                );
                if let Some(client) = session
                    .broker_clients
                    .values_mut()
                    .find(|c| c.app_name.as_deref() == Some("005-login"))
                {
                    let _ = client.end.send(&msg);
                }
            }
            let (bx, by) = twin_hit("005-login", "button:Submit").expect("005 Sign In 坐标");
            assert!(session.broker_pointer_down(ox + bx, oy + by, MouseButton::Left));
            assert!(
                wait_frames(&mut session, |s| composed_texts(s, "005-login")
                    .iter()
                    .any(|t| t.contains("Password is required"))),
                "005 提交后错误经 if 块显示"
            );
        }

        // ---- 收尾：Close 全部 → child 退出码 0。 ----
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

    /// 真 `auto` 二进制定位（缺则增量构建一次——首跑数秒，之后缓存命中）。
    /// independent 臂 = iced 主线程约束（winit Windows 事件循环须主线程，
    /// 测试 harness 线程必炸）→ 用生产二进制做 child（兼得三态 spawn 参数
    /// 全真链路）。
    fn auto_exe() -> std::path::PathBuf {
        let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
        for profile in ["debug", "release"] {
            let p = target.join(profile).join("auto.exe");
            if p.exists() {
                return p;
            }
        }
        let status = std::process::Command::new("cargo")
            .args(["build", "-p", "auto", "--bin", "auto"])
            .current_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
            .status()
            .expect("spawn cargo build -p auto");
        assert!(status.success(), "cargo build -p auto 失败");
        target.join("debug").join("auto.exe")
    }

    /// T3 主体二：independent 像素帧合成 + **双模并存**（同宿主一 queue
    /// 一 independent）：queue 臂 re-exec 测试二进制（真协议泵）；
    /// independent 臂 = 真 `auto run` 生产二进制（`--autodesk-render=
    /// independent` 三态参数 + iced 隐藏窗主线程约束）。断言两臂帧同达
    /// 宿主（DrawList / 像素前缓冲）。
    #[test]
    fn t3_independent_pixels_and_dual_mode() {
        let broker_pipe = format!("autodesk-broker-t3d-{}", std::process::id());
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        let src = example_source("001-helloworld");
        let known: Vec<&str> = vec!["001-helloworld"];
        session.desktop.app_resolver = Some(std::sync::Arc::new(move |name: &str| {
            known
                .iter()
                .find(|n| **n == name)
                .map(|n| LaunchSpec {
                    code: src.clone(),
                    source_path: None,
                    title: Some(n.to_string()),
                    name: None,
                    fit: false,
                    daemon: None,
                    back_root: None,
                })
        }));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));

        let mut queue_child = spawn_t3_child(&broker_pipe, "001-helloworld", "queue");
        // independent 臂 = 生产二进制（cwd = 仓根，examples/ui 相对解析）。
        let exe = auto_exe();
        let mut pixels_child = std::process::Command::new(&exe)
            .args([
                "run",
                "--autodesk-incubate",
                "--app386=001-helloworld",
                "--autodesk-render=independent",
                &format!("--autodesk-broker={broker_pipe}"),
            ])
            .current_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .expect("spawn auto independent child");

        attach_until(&mut session, 2);
        assert_eq!(
            session.broker_clients.values().filter_map(|c| c.wid).count(),
            2,
            "双 child 孵化落地"
        );
        // 双模并存：一 Commands 一 Pixels。
        let modes: Vec<_> =
            session.broker_clients.values().map(|c| c.endpoint.frame_mode).collect();
        assert!(modes.contains(&crate::ui::desktop_protocol::message::FrameMode::Commands));
        assert!(modes.contains(&crate::ui::desktop_protocol::message::FrameMode::Pixels));

        // queue 臂：DrawList 帧（"Hello, World!"）。
        assert!(
            wait_frames(&mut session, |s| {
                s.broker_clients.values().any(|c| {
                    c.endpoint.frame_mode
                        == crate::ui::desktop_protocol::message::FrameMode::Commands
                        && c.composed().is_some_and(|l| {
                            l.ops.iter().any(|op| matches!(op,
                                crate::ui::desktop_protocol::message::DrawOp::Text { text, .. }
                                    if text == "Hello, World!"))
                        })
                })
            }),
            "queue 臂 DrawList 帧到位"
        );
        // independent 臂：像素前缓冲（隐藏窗截图经降采样达宿主）。
        assert!(
            wait_frames(&mut session, |s| {
                s.broker_clients.values().any(|c| {
                    c.endpoint.frame_mode
                        == crate::ui::desktop_protocol::message::FrameMode::Pixels
                        && c.composed_pixels().is_some_and(|p| p.w > 0 && p.h > 0)
                })
            }),
            "independent 臂像素帧达宿主"
        );

        // 收尾。
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
        for child in [&mut queue_child, &mut pixels_child] {
            let start = std::time::Instant::now();
            let status = loop {
                if let Some(s) = child.try_wait().expect("try_wait") {
                    break s;
                }
                assert!(
                    start.elapsed() < std::time::Duration::from_secs(45),
                    "像素臂 child 收尾超时"
                );
                std::thread::sleep(std::time::Duration::from_millis(50));
            };
            assert!(status.success(), "child 退出码 {status}");
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = transport::connect(&broker_pipe, 500);
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
                        name: None,
                        daemon: None,
                        back_root: None,
                        fit: false,
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

    /// S9 L3 v2a 快照迁移：融合态 App（count=42）→ AutoVM 快照 →
    /// 孵化 child 注入恢复——composed 帧先同步出 count: 42，点击推进
    /// 43；revision 延续（快照 41 + 1 次点击 = 42）。
    #[test]
    fn stage3_l3_v2a_snapshot_migration() {
        use crate::ui::desktop_protocol::client_runtime::ClientPump;
        use crate::ui::desktop_protocol::host::ProtocolHost;
        use crate::ui::desktop_protocol::message::{ControlMsg, DrawOp, ProtocolMsg as PMsg};
        use crate::ui::session::DesktopSession;

        const SRC: &str = "widget MigCounter {
    model { var count int = 0 }
    view {
        button \"+\" { onclick: () => {.count += 1} }
        text `count: ${.count}`
    }
}
";

        // ---- 融合态 App：直挂组件状态推进（等价一段交互后的状态）。
        let mut fused = crate::build_dynamic_component(SRC, None).expect("fused build");
        fused
            .write_state("count", auto_val::Value::Int(42))
            .expect("write count");
        let payload = fused_state_snapshot(&fused, 41);

        // 载荷线格式恒等（追加式演进纪律：encode→decode 往返）。
        let wire = PMsg::Control(ControlMsg::StateSnapshot { wid: 1, payload: payload.clone() });
        let encoded = wire.encode();
        let decoded = PMsg::decode(&encoded).expect("decode");
        assert_eq!(decoded, wire, "StateSnapshot 线格式往返恒等");

        // ---- 孵化 child（真实管道对）并注入快照。
        let pipe = format!("autodesk-l3-s9-{}", std::process::id());
        let listener = transport::listen(&pipe).expect("listen");
        let config = ClientConfig {
            app_name: "mig-counter".into(),
            title: "mig".into(),
            width: 480.0,
            height: 320.0,
        };
        let app_end = transport::connect(&pipe, 2000).expect("connect");
        let mut client = ClientPump::new(
            app_end,
            {
                let component = crate::build_dynamic_component(SRC, None).expect("child build");
                AppProjector::new(component, 480.0, 320.0)
            },
            config,
            None,
        );
        let mut server_end = listener.wait_connect().expect("server");

        let mut session = DesktopSession::__test_session();
        session.__test_open_desktop();
        let src = SRC;
        let mut ph = ProtocolHost::new(&mut session, move |name: &str| {
            if name == "mig-counter" {
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

        // 泵到 Active + 首帧（此时 child 还在 count: 0）。
        let mut wid = None;
        for _ in 0..1000 {
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
        let wid = wid.expect("child 已孵化");

        // ---- 注入快照：host → StateSnapshot → child 应用 → 产帧同步。
        server_end
            .send(&PMsg::Control(ControlMsg::StateSnapshot { wid: wid.0, payload }))
            .unwrap();
        let mut injected = false;
        for _ in 0..1000 {
            pump(&mut server_end, &mut ph);
            let _ = client.step();
            if ph.composed(wid.0).is_some_and(|l| l.ops.iter().any(|op| matches!(op,
                DrawOp::Text { text, .. } if text == "count: 42"))) {
                injected = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(injected, "快照注入恢复：composed 帧同步出 count: 42（迁移前后一致）");

        // ---- 迁移后可交互：点击推进 43。
        let injected_click = ph.pointer_down(60.0, 40.0, MouseButton::Left).expect("窗内命中");
        server_end.send(&injected_click).unwrap();
        let mut clicked = false;
        for _ in 0..1000 {
            pump(&mut server_end, &mut ph);
            let _ = client.step();
            if ph.composed(wid.0).is_some_and(|l| l.ops.iter().any(|op| matches!(op,
                DrawOp::Text { text, .. } if text == "count: 43"))) {
                clicked = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(clicked, "迁移后点击推进 count: 43");

        // ---- revision 延续：快照 41 + 1 次点击 = 42。
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
        assert_eq!(projector.read_state("count").unwrap(), auto_val::Value::Int(43));
        assert_eq!(projector.revision(), 42, "revision 延续（快照 41 + 点击 1）");
    }

    /// Plan 500 步骤 5：宿主像素臂合成——HostEndpoint 收 FrameReadyPixels
    /// → ComposeFramePixels 动作 → BrokerClient::compose_pixels 读 shm 槽
    /// RGBA 入前缓冲 + FrameAck 回带（宽/高/stride/revision 元数据全链）。
    #[test]
    fn broker_pixels_compose_front_buffer() {
        use crate::ui::desktop_protocol::endpoint::{HostAction, HostEndpoint};
        use crate::ui::desktop_protocol::message::{
            FrameMsg, HandshakeMsg, PixelFormat, ProtocolMsg, WRect,
        };

        // 端点：Hello → activate(Pixels) → Active。
        let mut host = HostEndpoint::listen();
        let hello = {
            let mut app = super::super::endpoint::AppEndpoint::new(
                super::super::pixels::PixelsNoopSource::new(),
                "px",
                "px",
                32.0,
                16.0,
            );
            app.connect().expect("hello")
        };
        let actions = host.on_message(hello).expect("host 状态机");
        assert!(matches!(actions[0], HostAction::ResolveAndAttach { .. }));
        host.activate(1, 9, 77, WRect::new(0.0, 0.0, 32.0, 16.0), FrameMode::Pixels)
            .expect("activate");

        // child 侧写 shm 槽（32×16 纯色帧，槽尺寸 = 像素上限）。
        let shm_name = format!("autodesk-shm-px5-{}", std::process::id());
        let slot_size = super::super::pixels::pixels_slot_size(32.0, 16.0);
        let child_shm = SharedFrameBuffer::create(&shm_name, 2, slot_size).expect("child shm");
        let rgba: Vec<u8> = std::iter::repeat([7u8, 8, 9, 255])
            .take((32 * 16) as usize)
            .flatten()
            .collect();
        child_shm.write_slot(1, &rgba).expect("write slot");

        // 端点收 FrameReadyPixels → ComposeFramePixels。
        let actions = host
            .on_message(ProtocolMsg::Frame(FrameMsg::FrameReadyPixels {
                wid: 9,
                frame_id: 4,
                slot: 1,
                damage: None,
                revision: 12,
                w: 32,
                h: 16,
                stride: 128,
                format: PixelFormat::Rgba8,
            }))
            .expect("Active 收帧");
        let HostAction::ComposeFramePixels {
            surface, wid, frame_id, slot, revision, w, h, stride,
        } = &actions[0]
        else {
            panic!("期待 ComposeFramePixels: {actions:?}");
        };
        assert_eq!((*wid, *frame_id, *slot, *revision, *w, *h, *stride), (9, 4, 1, 12, 32, 16, 128));

        // BrokerClient 合成：宿主开同名段 → 前缓冲 + ack。
        let pipe = format!("autodesk-px5-pipe-{}", std::process::id());
        let listener = transport::listen(&pipe).expect("listen");
        let end = transport::connect(&pipe, 500).expect("connect");
        let mut client = BrokerClient::new(pipe, end);
        let host_shm = SharedFrameBuffer::open(&shm_name, 2, slot_size).expect("host shm");
        client.shm.insert(*surface, host_shm);
        client.wid = Some(Wid(9));
        client.wid_surface.insert(9, *surface);
        let ack = client.compose_pixels(
            *surface, *wid, *frame_id, *slot, *revision, *w, *h, *stride,
        )
        .expect("compose");
        assert_eq!(
            ack,
            FrameMsg::FrameAck { wid: 9, frame_id: 4, slot: 1 },
            "ack 回带 frame_id/槽"
        );
        let front = client.composed_pixels().expect("前缓冲在册");
        assert_eq!((front.w, front.h, front.stride, front.revision), (32, 16, 128, 12));
        assert_eq!(front.rgba, rgba, "槽字节 = child 写入帧");
        drop(listener);
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
                        name: None,
                        daemon: None,
                        back_root: None,
                        fit: false,
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

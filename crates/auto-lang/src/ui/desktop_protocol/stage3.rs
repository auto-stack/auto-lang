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

        // ---- spawn N 个 child（re-exec 测试二进制；剥离 NEXTEST_* 守护）。
        let exe = std::env::current_exe().expect("current_exe");
        let mut children = Vec::new();
        for app in &names {
            let mut cmd = std::process::Command::new(&exe);
            cmd.args(["stage3_child_body", "--test-threads", "1", "--nocapture"])
                .env(CHILD_BROKER_ENV, &broker_pipe)
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

        // ---- 孵化 attach 循环：直到 N 个全落地（30s 预算）。
        let mut landed: Vec<Wid> = Vec::new();
        let attach_deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while landed.len() < n && std::time::Instant::now() < attach_deadline {
            session.attach_pending_incubations(2000);
            session.pump_broker_clients();
            landed = session.broker_clients.values().filter_map(|c| c.wid).collect();
            if landed.len() == n {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
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
}

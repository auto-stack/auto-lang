// Plan 386 S12 —— 双模 exe 两进程集成测试（spawn-client 直连路径 ①）。
//
// 父测试 = 桌面侧：ProtocolHost 绑真实 462 会话 + `listen` per-app 管道；
// 子进程 = **测试二进制 re-exec**（`current_exe() + <child test> --exact`，
// 移除 NEXTEST_* 环境变量绕过 nextest 的直接运行保护）走双模 exe 全流程：
//
// `adjudicate`（①孵化标记 → Client）→ `connect` → AppEndpoint 握手 →
// Active → 共享内存帧（FrameReadyShared，S9：载荷不走管道）→ 协议点击 →
// `L2Detach` → Standalone（状态保持）→ 打印状态标记退出。
//
// 父进程断言：462 对象孵化落地、共享内存帧内容随点击递增、L2Detached →
// ReclaimWindow（窗/App/表面全清）、子进程 stdout 状态标记（**app 进程
// 持有状态**的跨进程证据）。broker 路径 ② 的全链已由 S10 覆盖，探测
// 逻辑由 broker::adjudicate 单测覆盖。

use super::broker;
use super::endpoint::{AppEndpoint, AppState, FrameSource};
use super::host::ProtocolHost;
use super::message::{ControlMsg, DrawList, FrameMsg, InputMsg, MouseButton, ProtocolMsg, Rgba8, WRect};
use super::shm::SharedFrameBuffer;
use super::transport::{self, Transport};
use crate::ui::dynamic::DynamicComponent;
use crate::ui::session::DesktopSession;

/// 子进程识别自身的环境键（值 = per-app 管道名）。
const CHILD_ENV: &str = "AUTO_386_CHILD_PIPE";
/// 子进程状态标记前缀（父进程 stdout 断言）。
const CHILD_MARKER: &str = "AUTO386-STANDALONE";

/// 计数器 demo 源码（单源；直挂/协议/spawn 三路同码）。
const COUNTER_SRC: &str = "widget SpawnCounter {\n    model { var count int = 0 }\n    view {\n        button \"+\" { onclick: () => {.count += 1} }\n        text `count: ${.count}`\n    }\n}\n";

/// app 侧窗口垫片（demo::CounterFrameSource 的两进程版——含共享内存产帧）。
struct SpawnCounterSource {
    component: DynamicComponent,
    button: WRect,
    rev: u64,
}

impl SpawnCounterSource {
    fn new(component: DynamicComponent) -> Self {
        Self { component, button: WRect::new(10.0, 10.0, 120.0, 36.0), rev: 1 }
    }

    fn count(&self) -> i64 {
        match self.component.read_state("count") {
            Ok(auto_val::Value::Int(n)) => n as i64,
            other => panic!("count: {other:?}"),
        }
    }
}

impl FrameSource for SpawnCounterSource {
    fn revision(&self) -> u64 {
        self.rev
    }

    fn render_frame(&mut self) -> DrawList {
        let n = self.count();
        let b = self.button;
        DrawList {
            clear: Some(Rgba8::new(24, 24, 28, 255)),
            ops: vec![
                super::message::DrawOp::Quad { rect: b, color: Rgba8::new(48, 96, 200, 255) },
                super::message::DrawOp::Text {
                    x: b.x + 50.0,
                    y: b.y + 10.0,
                    size: 14.0,
                    line_height: 18.0,
                    color: Rgba8::new(255, 255, 255, 255),
                    text: "+".into(),
                },
                super::message::DrawOp::Text {
                    x: 10.0,
                    y: 60.0,
                    size: 16.0,
                    line_height: 20.0,
                    color: Rgba8::new(220, 220, 220, 255),
                    text: format!("count: {n}"),
                },
            ],
        }
    }

    fn on_input(&mut self, input: &InputMsg) {
        if let InputMsg::PointerPressed { x, y, button: MouseButton::Left, .. } = input {
            let b = self.button;
            if *x >= b.x && *x < b.x + b.w && *y >= b.y && *y < b.y + b.h {
                self.component.on_with_input("__evt_onclick_1", None);
                self.rev += 1;
            }
        }
    }

    fn on_control(&mut self, _control: &ControlMsg) {}
}

/// 桌面侧泵（父进程）：app→host 处理 + 宿主应答回发。
fn pump_host(server_end: &mut Box<dyn Transport + Send>, ph: &mut ProtocolHost<'_>) {
    while let Some(loaded) = server_end.try_recv() {
        let msg = loaded.expect("解码");
        ph.handle(&msg).expect("host 状态机");
        for reply in std::mem::take(&mut ph.to_app) {
            // 对端（app 进程）在 Standalone 后可随时退出——BufferRelease
            // 等 best-effort 通知的发送失败不算错。
            let _ = server_end.send(&reply);
        }
    }
}

/// 父测试：spawn 子进程（双模 exe ① 路径）跑全生命周期。
#[test]
fn dual_mode_spawn_client_two_process() {
    // ---- 桌面侧就绪 ----
    let mut session = DesktopSession::__test_session();
    session.open_desktop(iced::window::Id::unique());
    let mut ph = ProtocolHost::new(&mut session, |name: &str| {
        if name == "counter" {
            crate::build_dynamic_component(COUNTER_SRC, None).map_err(|e| format!("{e}"))
        } else {
            Err(format!("unknown app {name}"))
        }
    });

    // per-app 管道先 listen（spawn 注入①：子进程凭标记直连）。
    let pipe = format!("autodesk-app-spawn-{}", std::process::id());
    let listener = transport::listen(&pipe).expect("listen");

    // ---- spawn 子进程（re-exec 测试二进制；剥离 NEXTEST_* 守护）----
    let exe = std::env::current_exe().expect("current_exe");
    let mut cmd = std::process::Command::new(exe);
    cmd.args(["dual_mode_child_body", "--test-threads", "1", "--nocapture"])
        .env("AUTO_386_CHILD_PIPE", &pipe)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit());
    for (k, _) in std::env::vars() {
        if k.starts_with("NEXTEST_") {
            cmd.env_remove(&k);
        }
    }
    let mut child = cmd.spawn().expect("spawn 子进程");

    // ---- 孵化：子进程连入 → 桌面侧端点 ----
    let mut server_end = listener.wait_connect().expect("server connect");

    // 泵到 Active（Hello → Welcome/BufferAlloc(shm) → Ready）。
    let mut wid = None;
    for _ in 0..100 {
        pump_host(&mut server_end, &mut ph);
        if !ph.session.apps.is_empty() {
            wid = ph.active().1;
            if ph.to_app.is_empty() {
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let wid = wid.expect("子进程已孵化").0;
    assert!(
        ph.session.apps.contains_key(&crate::ui::session::AppId(1)),
        "真实 462 AppSession 落地"
    );

    // ---- 共享内存帧：3 次协议点击 → 帧内容递增（载荷走 shm 不走管道）----
    for expect in 1..=3i64 {
        let injected = ph.pointer_down(60.0, 40.0, MouseButton::Left).expect("窗内命中");
        server_end.send(&injected).unwrap();
        let mut composed = None;
        for _ in 0..200 {
            pump_host(&mut server_end, &mut ph);
            if let Some(list) = ph.composed(wid) {
                let text = list.ops.iter().find_map(|op| match op {
                    super::message::DrawOp::Text { text, .. } if text.starts_with("count:") => {
                        Some(text.clone())
                    }
                    _ => None,
                });
                if text.as_deref() == Some(&format!("count: {expect}")) {
                    composed = text;
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(
            composed.as_deref(),
            Some(format!("count: {expect}").as_str()),
            "共享内存帧内容随点击递增"
        );
    }

    // ---- L2 独立出去：Detach → app Standalone → 确认 → 回收 ----
    let detach = ph.endpoint.l2_detach().expect("Active 才可 l2_detach");
    server_end.send(&detach).unwrap();
    for _ in 0..100 {
        pump_host(&mut server_end, &mut ph);
        if ph.session.apps.is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(ph.session.apps.is_empty(), "L2Detached 后 App 回收");
    assert!(ph.session.host.as_ref().unwrap().wm.wins.is_empty(), "虚拟窗回收");
    assert!(ph.surfaces.is_empty(), "表面释放");

    // ---- 等 child 退出 + 断言状态标记（app 进程持有状态的证据）----
    let start = std::time::Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().expect("try_wait") {
            break status;
        }
        assert!(
            start.elapsed() < std::time::Duration::from_secs(30),
            "子进程 30s 未退出"
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    };
    assert!(status.success(), "子进程退出码 {status}");
    let mut stdout = String::new();
    if let Some(mut out) = child.stdout.take() {
        use std::io::Read as _;
        let _ = out.read_to_string(&mut stdout);
    }
    assert!(
        stdout.contains(&format!("{CHILD_MARKER} count=3 rev=4")),
        "子进程状态标记（count=3 次点击、rev=1+3 状态保持）: {stdout:?}"
    );
}

/// 子进程体：双模 exe 的"client 态"全流程（由父测试 spawn 运行；
/// 直接跑套件时无 `AUTO_386_CHILD_PIPE` 环境则跳过）。
#[test]
fn dual_mode_child_body() {
    let Ok(pipe) = std::env::var(CHILD_ENV) else {
        return;
    };

    // ① 入口裁决：孵化标记 → Client（真函数、合成 argv）。
    let args = vec![format!("--autodesk-client={pipe}")];
    assert_eq!(
        broker::adjudicate(&args, 100),
        broker::EntryPoint::Client { pipe: pipe.clone() }
    );

    // 直连 per-app 管道 → AppEndpoint（真实组件）。
    let mut app_end = transport::connect(&pipe, 2000).expect("connect");
    let source = SpawnCounterSource::new(
        crate::build_dynamic_component(COUNTER_SRC, None).expect("build"),
    );
    let mut app = AppEndpoint::new(source, "counter", "计数器", 480.0, 320.0);

    // 握手 → Active；BufferAlloc.shm → 打开共享内存段。
    let hello = app.connect().unwrap();
    app_end.send(&hello).unwrap();
    let mut shm: Option<SharedFrameBuffer> = None;
    for _ in 0..200 {
        if let Some(loaded) = app_end.recv_wait(50) {
            let msg = loaded.expect("解码");
            let mut shm_name: Option<String> = None;
            if let ProtocolMsg::Frame(FrameMsg::BufferAlloc { shm: Some(name), .. }) = &msg {
                shm_name = Some(name.clone());
            }
            for reply in app.on_message(msg).expect("app 状态机") {
                app_end.send(&reply).unwrap();
            }
            if let Some(name) = shm_name {
                shm = Some(SharedFrameBuffer::open(&name, 2, 16384).expect("open shm"));
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(app.state, AppState::Active, "握手完成");
    let shm = shm.expect("BufferAlloc 已携带 shm 名");

    // 主循环：处理输入 → 共享内存产帧；L2Detach → Standalone 标记退出。
    loop {
        let Some(Ok(msg)) = app_end.recv_wait(500) else {
            continue;
        };
        match &msg {
            ProtocolMsg::Input(_) => {
                app.on_message(msg).unwrap();
                let frame = app.produce_frame_shared(&shm, None).expect("产帧");
                app_end.send(&frame).unwrap();
            }
            ProtocolMsg::Control(ControlMsg::L2Detach { .. }) => {
                let replies = app.on_message(msg).unwrap();
                assert_eq!(app.state, AppState::Standalone);
                for reply in replies {
                    app_end.send(&reply).unwrap();
                }
                println!(
                    "{CHILD_MARKER} count={} rev={}",
                    app.session.count(),
                    app.session.revision()
                );
                break;
            }
            _ => {
                app.on_message(msg).unwrap();
            }
        }
    }
}

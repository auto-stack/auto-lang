// Plan 386 S10 —— AutoDesk broker 与双模 exe 入口裁决（autoshell §7.1）。
//
// 入口裁决三步（[`adjudicate`]）：
// ① 命令行带孵化标记 `--autodesk-client=<pipe>`（桌面 spawn 注入）→ 连入；
// ② 无标记但 broker 在线（`\\.\pipe\autodesk-broker` 探测握手）→ 连入；
// ③ 都没有 → 独立模式自开 OS 窗（= `auto run` 现行为）。
//
// broker 语义：app 连上 broker 管道 → 上报 app 名（DesktopBus 同形记录
// `incubate\u{1F}<name>`）→ broker 分配 per-app 管道名（并先行 listen）
// → 回名（`incubate\u{1F}<pipe>`）→ app 转连 → 此后正常桌面协议握手。
// 空连接（探测 ping：连上即关）被 serve 循环识别吞掉——探测不影响
// broker 存活，也不泄漏孵化名额。管道名可参数化（生产 = [`BROKER_PIPE`]，
// 测试 = pid 后缀防串扰）。

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use super::message::{ControlMsg, ProtocolMsg};
use super::transport::{self, Transport, TransportError};

/// broker 管道名（autoshell §7.1 定名）。
pub const BROKER_PIPE: &str = "autodesk-broker";

/// 双模 exe 入口裁决结果。
#[derive(Debug, Clone, PartialEq)]
pub enum EntryPoint {
    /// ① spawn 注入标记：直接连指定管道（跳过 broker）。
    Client { pipe: String },
    /// ② broker 在线：经 broker 请求孵化。
    Broker,
    /// ③ 独立模式：自开 OS 窗（= `auto run` 现行为）。
    Standalone,
}

/// 入口裁决三步（autoshell §7.1 定案顺序，不可换）。生产入口——固定
/// [`BROKER_PIPE`] 探测（委托 [`adjudicate_on`]）。
pub fn adjudicate(args: &[String], broker_probe_timeout_ms: u32) -> EntryPoint {
    adjudicate_on(BROKER_PIPE, args, broker_probe_timeout_ms)
}

/// Plan 489：adjudicate 的参数化管道版（可测性缝，`Broker::on_pipe` 同型）
/// ——测试用 pid 后缀管道探测，摆脱对生产固定管道全局命名空间状态的依赖
/// （本机任何桌面宿主 listen 固定管道时，原测试步骤③ 的 Standalone
/// 断言被打穿——P487-2 间歇红根因）。生产行为零变化。
pub fn adjudicate_on(
    pipe: &str,
    args: &[String],
    broker_probe_timeout_ms: u32,
) -> EntryPoint {
    // ① 孵化标记。
    for arg in args {
        if let Some(pipe) = arg.strip_prefix("--autodesk-client=") {
            return EntryPoint::Client { pipe: pipe.to_string() };
        }
    }
    // ② broker 探测（连上即关 = ping；broker 侧吞空连接）。
    if transport::connect(pipe, broker_probe_timeout_ms).is_ok() {
        return EntryPoint::Broker;
    }
    // ③ 独立。
    EntryPoint::Standalone
}

/// broker 服务（孵化名额分配 + per-app 管道中转）。
pub struct Broker {
    pipe_name: String,
    next_id: AtomicU64,
    stopped: Arc<AtomicBool>,
}

impl Broker {
    /// 生产构造（固定 [`BROKER_PIPE`]）。
    pub fn new() -> Self {
        Self::on_pipe(BROKER_PIPE.to_string())
    }

    /// 指定管道名构造（测试 pid 后缀防串扰）。
    pub fn on_pipe(pipe_name: String) -> Self {
        Self { pipe_name, next_id: AtomicU64::new(0), stopped: Arc::new(AtomicBool::new(false)) }
    }

    pub fn stop_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.stopped)
    }

    /// 接一连接：空连接（探测 ping）返回 Ok(None)；真实孵化请求返回
    /// (per-app 管道名, 桌面侧协议端点)。阻塞——调用方放独立线程。
    pub fn serve_once(&mut self) -> Result<Option<(String, Box<dyn Transport + Send>)>, TransportError> {
        let listener = transport::listen(&self.pipe_name)?;
        let mut client = listener.wait_connect()?;
        // 读孵化请求；100ms 无请求 = 探测 ping（连上即关）→ 吞掉重听。
        let request = match client.recv_wait(100) {
            Some(Ok(msg)) => msg,
            _ => return Ok(None),
        };
        let app_name = match &request {
            ProtocolMsg::Control(ControlMsg::DesktopBus { record, .. }) => record
                .split_once('\u{1f}')
                .filter(|(verb, _)| *verb == "incubate")
                .map(|(_, name)| name.to_string())
                .ok_or_else(|| TransportError::Io("bad incubate record".into()))?,
            _ => return Ok(None),
        };
        let _ = &app_name; // 名字进日志/注册表归 Stage 2 桌面壳；v1 仅分配
        let n = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        let pipe_name = format!("{}-app-{n}", self.pipe_name);
        let app_listener = transport::listen(&pipe_name)?;
        let reply = ProtocolMsg::Control(ControlMsg::DesktopBus {
            wid: 0,
            record: format!("incubate\u{1f}{pipe_name}"),
        });
        client.send(&reply)?;
        let end = app_listener.wait_connect()?;
        Ok(Some((pipe_name, end)))
    }
}

/// app 侧孵化请求：连 broker → 上报 app 名 → 收 per-app 管道名 → 转连。
/// 返回 (per-app 管道名, app 侧协议端点)。
pub fn request_incubation(
    broker_pipe: &str,
    app_name: &str,
    timeout_ms: u32,
) -> Result<(String, Box<dyn Transport + Send>), TransportError> {
    let mut broker_end = transport::connect(broker_pipe, timeout_ms)?;
    let ask = ProtocolMsg::Control(ControlMsg::DesktopBus {
        wid: 0,
        record: format!("incubate\u{1f}{app_name}"),
    });
    broker_end.send(&ask)?;
    let reply = broker_end
        .recv_wait(timeout_ms)
        .ok_or(TransportError::Eof)?
        .map_err(TransportError::Codec)?;
    let pipe_name = match reply {
        ProtocolMsg::Control(ControlMsg::DesktopBus { record, .. }) => record
            .split_once('\u{1f}')
            .filter(|(verb, _)| *verb == "incubate")
            .map(|(_, name)| name.to_string())
            .ok_or_else(|| TransportError::Io("bad incubate reply".into()))?,
        _ => return Err(TransportError::Io("bad incubate reply".into())),
    };
    let end = transport::connect(&pipe_name, timeout_ms)?;
    Ok((pipe_name, end))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::desktop_protocol::endpoint::{AppEndpoint, AppState, FrameSource};
    use crate::ui::desktop_protocol::host::ProtocolHost;
    use crate::ui::desktop_protocol::message::{
        DrawList, FrameMsg, HandshakeMsg, InputMsg, MouseButton,
    };
    use crate::ui::dynamic::DynamicComponent;
    use crate::ui::session::{DesktopSession, Wid};

    /// 计数器 FrameSource（demo 同款最小内联版）。
    struct CounterSource {
        component: DynamicComponent,
        rev: u64,
    }

    impl CounterSource {
        fn new(component: DynamicComponent) -> Self {
            Self { component, rev: 1 }
        }
        fn count(&self) -> i64 {
            match self.component.read_state("count") {
                Ok(auto_val::Value::Int(n)) => n as i64,
                other => panic!("count: {other:?}"),
            }
        }
    }

    impl FrameSource for CounterSource {
        fn revision(&self) -> u64 {
            self.rev
        }
        fn render_frame(&mut self) -> DrawList {
            DrawList::default()
        }
        fn on_input(&mut self, input: &InputMsg) {
            if matches!(input, InputMsg::PointerPressed { button: MouseButton::Left, .. }) {
                self.component.on_with_input("__evt_onclick_1", None);
                self.rev += 1;
            }
        }
        fn on_control(&mut self, _c: &ControlMsg) {}
    }

    const SRC: &str = "widget BrokerCounter {\n    model { var count int = 0 }\n    view {\n        button \"+\" { onclick: () => {.count += 1} }\n    }\n}\n";

    fn resolver() -> impl FnMut(&str) -> Result<DynamicComponent, String> {
        |name: &str| {
            if name == "counter" {
                crate::build_dynamic_component(SRC, None).map_err(|e| format!("{e}"))
            } else {
                Err(format!("unknown app {name}"))
            }
        }
    }

    /// 有界双向泵一轮：app→host（含宿主应答回发）+ host→app（含端点
    /// 应答回发）。管道交付异步，各方向用 recv_wait 有界等待。
    fn pump_pair(
        server_end: &mut Box<dyn Transport + Send>,
        app_end: &mut Box<dyn Transport + Send>,
        app: &mut AppEndpoint<CounterSource>,
        ph: &mut ProtocolHost<'_>,
    ) {
        // app → host。
        while let Some(loaded) = server_end.try_recv() {
            let msg = loaded.unwrap();
            ph.handle(&msg).unwrap();
            for reply in std::mem::take(&mut ph.to_app) {
                server_end.send(&reply).unwrap();
            }
        }
        // host → app。
        while let Some(loaded) = app_end.recv_wait(50) {
            let msg = loaded.unwrap();
            for reply in app.on_message(msg).unwrap() {
                app_end.send(&reply).unwrap();
            }
        }
        // app 的应答（Ready 等）再泵一轮宿主。
        while let Some(loaded) = server_end.try_recv() {
            let msg = loaded.unwrap();
            ph.handle(&msg).unwrap();
            for reply in std::mem::take(&mut ph.to_app) {
                server_end.send(&reply).unwrap();
            }
        }
    }

    #[test]
    fn adjudicate_three_steps() {
        // Plan 489：全程 pid 后缀管道（adjudicate_on 缝）——原测试②③ 探测
        // 生产固定管道，本机任何桌面宿主（或并行测试窗口期）listen 即打穿
        // ③ 的 Standalone 断言（P487-2 间歇红）。
        let pipe = format!("autodesk-broker-adjud-{}", std::process::id());
        // ① 孵化标记优先（无论 broker 是否在线）。
        let args = vec!["--autodesk-client=autodesk-app-7".to_string()];
        assert_eq!(
            adjudicate_on(&pipe, &args, 10),
            EntryPoint::Client { pipe: "autodesk-app-7".into() }
        );
        // ③ 无标记无 broker → Standalone（pid 管道无人 listen → 秒失败）。
        assert_eq!(adjudicate_on(&pipe, &[], 30), EntryPoint::Standalone);
        // ② broker 在线（serve 循环吞探测 ping）→ Broker。
        let mut broker = Broker::on_pipe(pipe.clone());
        let stop = broker.stop_flag();
        let stop2 = Arc::clone(&stop);
        let worker = std::thread::spawn(move || {
            while !stop2.load(Ordering::Relaxed) {
                let _ = broker.serve_once();
            }
        });
        assert_eq!(adjudicate_on(&pipe, &[], 2000), EntryPoint::Broker);
        stop.store(true, Ordering::Relaxed);
        // 空连接唤醒阻塞中的 serve 循环令其退出。
        let _ = transport::connect(&pipe, 500);
        let _ = worker.join();
    }

    #[test]
    fn broker_incubation_full_flow() {
        // broker 管道名 pid 后缀：防并行测试进程串扰。
        let broker_pipe = format!("autodesk-broker-test-{}", std::process::id());
        let mut broker = Broker::on_pipe(broker_pipe.clone());
        let host_side = std::thread::spawn(move || {
            broker.serve_once().unwrap().expect("孵化连接")
        });

        // app 侧：经 broker 请求孵化 → per-app 管道双端。
        let (pipe_name, mut app_end) = request_incubation(&broker_pipe, "counter", 2000).unwrap();
        assert!(pipe_name.contains("-app-"));

        let (server_pipe, mut server_end) = host_side.join().unwrap();
        assert_eq!(server_pipe, pipe_name);

        // 桌面侧：真实 462 会话 + ProtocolHost。
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        let mut ph = ProtocolHost::new(&mut session, resolver());

        // app 端点（真实组件 + 计数器 FrameSource）。
        let source =
            CounterSource::new(crate::build_dynamic_component(SRC, None).expect("build"));
        let mut app = AppEndpoint::new(source, "counter", "计数器", 480.0, 320.0);

        // 孵化握手：Hello → Welcome/BufferAlloc → Ready → Active
        //（异步交付：泵到收敛）。
        let hello = app.connect().unwrap();
        app_end.send(&hello).unwrap();
        for _ in 0..100 {
            pump_pair(&mut server_end, &mut app_end, &mut app, &mut ph);
            if app.state == AppState::Active {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(app.state, AppState::Active);
        assert_eq!(
            ph.session.apps.len(),
            1,
            "真实 462 AppSession 已登记"
        );
        assert!(ph
            .session
            .host
            .as_ref()
            .unwrap()
            .wm
            .wins
            .contains_key(&Wid(1)));

        // 协议点击一记：桌面 hit_test → (Wid,event) → VM handler。
        let injected = ph.pointer_down(60.0, 40.0, MouseButton::Left).unwrap();
        server_end.send(&injected).unwrap();
        for _ in 0..100 {
            pump_pair(&mut server_end, &mut app_end, &mut app, &mut ph);
            if app.session.count() == 1 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(app.session.count(), 1, "broker 孵化的 app 可输入");
        let _ = (FrameMsg::CacheControl { wid: 0, drop_keys: vec![] }, HandshakeMsg::Ready);
    }

    /// Plan 480 S3：`DesktopSession::enable_broker` 桌面模式集成——serve
    /// 线程受理 `request_incubation`，`attach_pending_incubations` 经
    /// ProtocolHost ResolveAndAttach 路径落真实 462 会话（App + 虚拟窗 +
    /// 表面），注册表 resolver 按 app 名编译装载。
    #[test]
    fn enable_broker_incubates_into_real_session() {
        use crate::ui::session::{AppId, DesktopSession};
        use std::sync::atomic::AtomicBool;

        const SRC: &str = "widget Broker386 { model { var count int = 0 } view { text `c: ${.count}` } }\n";
        let broker_pipe = format!("autodesk-broker-s3-{}", std::process::id());

        // 桌面侧：真实 462 会话 + 注册表 resolver + enable_broker。
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        let code = SRC.to_string();
        session.desktop.app_resolver =
            Some(std::sync::Arc::new(move |name: &str| {
                if name == "broker386" {
                    Some(crate::ui::session::LaunchSpec {
                        code: code.clone(),
                        source_path: None,
                        title: Some("Broker386".into()),
                    })
                } else {
                    None
                }
            }));
        let stop = Arc::new(AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));

        // app 侧（同进程模拟 child）：request_incubation 双端连通。
        let (pipe_name, mut app_end) =
            request_incubation(&broker_pipe, "broker386", 5000).unwrap();
        assert!(pipe_name.contains("-app-"), "per-app 管道名 {pipe_name}");

        // 端点握手材料：Hello 发出后，attach 泵宿主侧到 Active。
        let src = SRC;
        let component = crate::build_dynamic_component(src, None).unwrap();
        let mut app = AppEndpoint::new(
            CounterSource::new(component),
            "broker386",
            "Broker386",
            480.0,
            320.0,
        );
        app_end.send(&app.connect().unwrap()).unwrap();

        // attach：等待 serve 线程转交 → 泵到 Active → 462 对象落地。
        let mut wid = None;
        for _ in 0..200 {
            let landed = session.attach_pending_incubations(2000);
            if let Some(w) = landed.first() {
                wid = Some(*w);
                break;
            }
            // app 侧消费 Welcome/BufferAlloc（attach 回发在端上）。
            while let Some(loaded) = app_end.recv_wait(5) {
                let _ = app.on_message(loaded.expect("解码"));
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let wid = wid.expect("孵化落地虚拟窗");
        assert!(session.apps.contains_key(&AppId(1)), "AppSession 已登记");
        assert!(
            session.host.as_ref().unwrap().wm.wins.contains_key(&wid),
            "虚拟窗已登记"
        );

        // 停机：置位 + probe 连接唤醒阻塞中的 serve 循环。
        stop.store(true, Ordering::Relaxed);
        let _ = transport::connect(&broker_pipe, 500);
    }
}

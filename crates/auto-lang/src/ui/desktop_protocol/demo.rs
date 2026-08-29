// Plan 386 Stage 1 —— loopback demo：一个 App 的帧/输入/控制经协议通路
// "渲染"进 462 虚拟窗口，行为与直挂无差（§0 Stage 1 验收句）。
//
// 两条通路的定义：
// - **协议路径**（被验证对象）：桌面 `WmState::hit_test` → (Wid, event)
//   过线（loopback 字节管道）→ `AppEndpoint` → `DynamicComponent::
//   on_with_input` —— 与 Stage 2 两进程后 app 收到的事件流同构。
// - **直挂对照**（现行路径）：同源码 `build_dynamic_component` 后直接
//   `on_with_input`（in-process 派发的 post-hit-test 语义）。
//
// 无差的判据：同一点击序列后 `read_state` 相等，且双方 `FrameSource`
// 产出的 `DrawList` 相等（帧是状态的投影，状态同则帧同）。
// 渲染落点：宿主 `SurfaceStore::front(wid)` 即虚拟窗合成面。

use crate::ui::desktop_protocol::endpoint::{AppEndpoint, AppState, FrameSource};
use crate::ui::desktop_protocol::host::ProtocolHost;
use crate::ui::desktop_protocol::loopback::{loopback_pair, LoopbackEnd};
use crate::ui::desktop_protocol::message::{
    ControlMsg, DrawList, DrawOp, InputMsg, MouseButton, ObserveMsg, ProtocolMsg, Rgba8,
};
use crate::ui::dynamic::DynamicComponent;
use crate::ui::session::DesktopSession;

/// 计数器 demo App（单源：协议路径与直挂对照用同一份源码编译）。
pub const COUNTER_SRC: &str = "widget ProtoCounter {\n    model { var count int = 0 }\n    view {\n        button \"+\" { onclick: () => {.count += 1} }\n        text `count: ${.count}`\n    }\n}\n";

/// app 侧窗口垫片 v1（计数器 demo）：协议输入 → widget 命中 → VM handler；
/// 会话状态 → [`DrawList`]。Stage 2 里这个垫片就是独立 exe 的 app 进程
/// 工具层（此处演示其与协议端点的最小接缝）。
pub struct CounterFrameSource {
    pub component: DynamicComponent,
    /// "+" 按钮的 widget 本地命中区。
    pub button: crate::ui::desktop_protocol::message::WRect,
    /// 命中按钮后派发的 handler 键。内联 lambda 在编译期按声明序铸造为
    /// `__evt_<event>_<n>`（demo 源只有一号按钮）；live 路径由渲染器在
    /// 视图构建期把同一键烘焙进按钮消息——垫片持有的就是这份映射。
    pub handler_key: &'static str,
    rev: u64,
}

impl CounterFrameSource {
    pub fn new(component: DynamicComponent) -> Self {
        Self {
            component,
            button: crate::ui::desktop_protocol::message::WRect::new(10.0, 10.0, 120.0, 36.0),
            handler_key: "__evt_onclick_1",
            rev: 1,
        }
    }

    /// 当前计数（VM 状态读取；协议路径与直挂的无差判据）。
    pub fn count(&self) -> i64 {
        match self.component.read_state("count") {
            Ok(auto_val::Value::Int(n)) => n as i64,
            other => panic!("count 读取失败: {other:?}"),
        }
    }
}

impl FrameSource for CounterFrameSource {
    fn revision(&self) -> u64 {
        self.rev
    }

    fn render_frame(&mut self) -> DrawList {
        let n = self.count();
        let b = self.button;
        DrawList {
            clear: Some(Rgba8::new(24, 24, 28, 255)),
            ops: vec![
                DrawOp::Quad { rect: b, color: Rgba8::new(48, 96, 200, 255) },
                DrawOp::Text {
                    x: b.x + 50.0,
                    y: b.y + 10.0,
                    size: 14.0,
                    line_height: 18.0,
                    color: Rgba8::new(255, 255, 255, 255),
                    text: "+".into(),
                },
                DrawOp::Text {
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
                // 与直挂完全同一调用点：DynamicComponent::on_with_input。
                self.component.on_with_input(self.handler_key, None);
                self.rev += 1;
            }
        }
    }

    fn on_control(&mut self, _control: &ControlMsg) {}
}

/// demo 运行报告（`run_counter_loopback` 的产物，测试断言/展示两用）。
#[derive(Debug, Clone)]
pub struct CounterDemoReport {
    pub wid: u64,
    pub surface: u64,
    /// 每个阶段后虚拟窗合成面（帧 0 → N 次点击后的帧）。
    pub composed: Vec<DrawList>,
    /// 协议路径终态计数。
    pub protocol_count: i64,
    /// 直挂对照终态计数。
    pub direct_count: i64,
    /// 帧投影无差（最后一帧 == 直挂侧 FrameSource 产出）。
    pub frame_parity: bool,
    /// VM 状态无差（read_state 相等）。
    pub state_parity: bool,
    /// 控制通道 Close → 虚拟窗回收是否完成。
    pub reclaimed: bool,
    /// 观测上行到达宿主的日志。
    pub observed_logs: Vec<String>,
    /// DesktopBus 上行到达宿主的记录。
    pub desktop_bus_records: Vec<String>,
}

/// 宿主侧泵：收 app 方消息 → 协议宿主处理 → 回发全部排队消息。
fn pump_host(host_sock: &mut LoopbackEnd, ph: &mut ProtocolHost<'_>) {
    while let Some(loaded) = host_sock.try_recv() {
        let msg = loaded.expect("loopback 解码");
        ph.handle(&msg).expect("host 状态机");
        for reply in std::mem::take(&mut ph.to_app) {
            host_sock.send(&reply);
        }
    }
}

/// app 侧泵：收宿主方消息 → app 端点处理 → 回发应答。
fn pump_app(app_sock: &mut LoopbackEnd, app: &mut AppEndpoint<CounterFrameSource>) {
    while let Some(loaded) = app_sock.try_recv() {
        let msg = loaded.expect("loopback 解码");
        for reply in app.on_message(msg).expect("app 状态机") {
            app_sock.send(&reply);
        }
    }
}

/// 跑一遍完整 loopback demo：孵化 → 帧 0 → `clicks` 次协议点击（每次
/// 点击后产帧）→ 观测 → DesktopBus → 控制 Close → 回收。返回全程报告。
pub fn run_counter_loopback(clicks: usize) -> CounterDemoReport {
    // --- 直挂对照（先行构建，同一份源码）---
    let mut direct = CounterFrameSource::new(
        crate::build_dynamic_component(COUNTER_SRC, None).expect("直挂侧编译"),
    );

    // --- 桌面 + 协议宿主 ---
    let mut session = DesktopSession::__test_session();
    session.open_desktop(iced::window::Id::unique());
    let mut ph = ProtocolHost::new(&mut session, |name: &str| {
        if name == "counter" {
            crate::build_dynamic_component(COUNTER_SRC, None)
                .map_err(|e| format!("build failed: {e}"))
        } else {
            Err(format!("unknown app: {name}"))
        }
    });

    // --- 协议路径的 app 端点 + 管道 ---
    let protocol_source = CounterFrameSource::new(
        crate::build_dynamic_component(COUNTER_SRC, None).expect("协议侧编译"),
    );
    let mut app = AppEndpoint::new(protocol_source, "counter", "计数器", 480.0, 320.0);
    let (mut app_sock, mut host_sock) = loopback_pair();

    // ① 孵化握手：Hello → Welcome/BufferAlloc → Ready。
    let hello = app.connect().expect("Detached 才可 connect");
    app_sock.send(&hello);
    pump_host(&mut host_sock, &mut ph);
    pump_app(&mut app_sock, &mut app);
    pump_host(&mut host_sock, &mut ph); // Ready 到宿主（例行收尾）
    assert_eq!(app.state, AppState::Active, "握手完成");
    let wid = app.wid.expect("Active 即有 wid");
    let surface = app.surface.expect("Active 即有 surface");

    // ② 帧 0：初渲染上屏。
    let f0 = app.produce_frame(None).expect("Active 产帧");
    app_sock.send(&f0);
    pump_host(&mut host_sock, &mut ph);
    pump_app(&mut app_sock, &mut app);
    let mut composed = Vec::new();
    composed.push(ph.composed(wid).cloned().expect("帧 0 已合成"));

    // ③ 协议点击 ×N：桌面 hit_test → (Wid,event) → VM handler → 新帧。
    for _ in 0..clicks {
        // 点击窗内 (60, 40)（全局）→ 本地 (44, 24)，落在按钮区 (10,10,120,36)。
        let injected = ph.pointer_down(60.0, 40.0, MouseButton::Left).expect("窗内命中");
        host_sock.send(&injected);
        pump_app(&mut app_sock, &mut app);
        let frame = app.produce_frame(None).expect("输入后产帧");
        app_sock.send(&frame);
        pump_host(&mut host_sock, &mut ph);
        pump_app(&mut app_sock, &mut app);
        composed.push(ph.composed(wid).cloned().expect("点击后帧已合成"));
        // 直挂对照：同一 handler 调用。
        direct.component.on_with_input("__evt_onclick_1", None);
        direct.rev += 1;
    }

    // ④ 观测：宿主接汇 → app 上行一条日志。
    let attach = ProtocolMsg::Observe(ObserveMsg::Attach { wid, sink: "mcp://desktop/app-1".into() });
    host_sock.send(&attach);
    pump_app(&mut app_sock, &mut app);
    let log = ProtocolMsg::Observe(ObserveMsg::Log {
        wid,
        level: crate::ui::desktop_protocol::message::LogLevel::Info,
        message: format!("count={}", app.session.count()),
    });
    app_sock.send(&log);
    pump_host(&mut host_sock, &mut ph);

    // ⑤ DesktopBus 上行（控制通道载荷 = DesktopCommand 同格式记录）。
    let bus = ProtocolMsg::Control(ControlMsg::DesktopBus {
        wid,
        record: "launch\u{1f}counter".into(),
    });
    app_sock.send(&bus);
    pump_host(&mut host_sock, &mut ph);

    // ⑥ 控制收尾：宿主 Close → app ExitRequest → 回收 → Detached。
    let close = ph.endpoint.close().expect("Active 才可 close");
    host_sock.send(&close);
    pump_app(&mut app_sock, &mut app);
    pump_host(&mut host_sock, &mut ph);
    pump_app(&mut app_sock, &mut app);
    let reclaimed = ph.session.apps.is_empty()
        && ph.session.host.as_ref().expect("desktop 模式").wm.wins.is_empty()
        && ph.surfaces.is_empty();

    // --- 无差判据 ---
    let protocol_count = app.session.count();
    let direct_count = direct.count();
    let direct_frame = direct.render_frame();
    let last_composed = composed.last().cloned().unwrap_or_default();
    let observed_logs = ph
        .observe_inbox
        .iter()
        .filter_map(|m| match m {
            ObserveMsg::Log { message, .. } => Some(message.clone()),
            _ => None,
        })
        .collect();
    let desktop_bus_records = ph
        .control_inbox
        .iter()
        .filter_map(|m| match m {
            ControlMsg::DesktopBus { record, .. } => Some(record.clone()),
            _ => None,
        })
        .collect();

    CounterDemoReport {
        wid,
        surface,
        composed,
        protocol_count,
        direct_count,
        frame_parity: last_composed == direct_frame,
        state_parity: protocol_count == direct_count,
        reclaimed,
        observed_logs,
        desktop_bus_records,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stage 1 验收：帧/输入/控制经协议通路与直挂无差。
    #[test]
    fn counter_loopback_demo_parity_with_direct_mount() {
        let report = run_counter_loopback(3);

        // 孵化：真实 462 对象获得句柄。
        assert_eq!(report.wid, 1, "首个虚拟窗 Wid(1)");
        assert_eq!(report.surface, 1);

        // 帧：帧 0 = "count: 0"，三次点击后 = "count: 3"。
        assert_eq!(report.composed.len(), 4, "帧 0 + 每击一帧");
        let text_of = |ops: &[DrawOp]| -> String {
            ops.iter()
                .find_map(|op| match op {
                    DrawOp::Text { text, .. } if text.starts_with("count:") => {
                        Some(text.clone())
                    }
                    _ => None,
                })
                .expect("count 文本存在")
        };
        assert_eq!(text_of(&report.composed[0].ops), "count: 0");
        assert_eq!(text_of(&report.composed.last().unwrap().ops), "count: 3");
        assert_eq!(
            report.composed[0].clear,
            Some(Rgba8::new(24, 24, 28, 255)),
            "清屏色随帧过线"
        );

        // 输入：协议路径与直挂的 VM 状态无差。
        assert_eq!(report.protocol_count, 3);
        assert_eq!(report.direct_count, 3);
        assert!(report.state_parity, "read_state 无差");

        // 帧投影无差。
        assert!(report.frame_parity, "最后一帧 == 直挂侧产出");

        // 控制：Close → 虚拟窗/App/表面全部回收（462 Close 语义）。
        assert!(report.reclaimed);

        // 观测与 DesktopBus 上行到达宿主。
        assert_eq!(report.observed_logs, vec!["count=3".to_string()]);
        assert_eq!(report.desktop_bus_records, vec!["launch\u{1f}counter".to_string()]);
    }

    /// 零点击退化路径：握手 + 帧 0 + 回收照常。
    #[test]
    fn counter_loopback_zero_clicks_still_complete() {
        let report = run_counter_loopback(0);
        assert_eq!(report.composed.len(), 1);
        assert_eq!(report.protocol_count, 0);
        assert_eq!(report.direct_count, 0);
        assert!(report.state_parity && report.frame_parity && report.reclaimed);
    }
}


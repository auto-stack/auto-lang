// PLAN-760 T-03 —— remote-frame 宿主客户端注册表 + canvas 画师。
//
// 元件消费面（`remote-frame { url }` 的 iced 渲染后端）：
// - 注册表（进程级 LazyLock）：url → Client。渲染臂每帧 `note(url)` 续租
//   （租约法 = 计划 §5.2 引用计数的等价稳健实现——url 消失/切换 = 租约
//   到期 → Client Drop → WsEnd Drop（shutdown oneshot + join 读线程，
//   transport.rs 既有语义，T-01 核证）——迟到帧随旧对象丢弃，身份防串）。
// - 泵（`pump()`，ServiceTick 400ms 拍挂 renderer.rs pump_remote_mirrors
//   之后）：非阻塞 try_recv 全排空 → Welcome（订阅兑现，记远端逻辑域）/
//   FrameReadyV2（最新帧槽）；EOF/协议错 → 重连预算内重拨（ReconnectPolicy
//   budget_ms/interval_ms 语义内联——client_runtime 的 struct 自带 pipe
//   字段不适用），预算尽 → 占位诊断终态。
// - 连接在独立线程完成（ws::connect 阻塞循环，不能占渲染线程）；结果经
//   结果槽回写，pump 收编后发 Hello 订阅。
// - 输入：canvas Program::update 臂（指针 move/press/release，bounds 局部
//   坐标 → 远端逻辑域等比换算）→ `ProtocolMsg::Input(InputMsg)` 直发
//   （rqhost.rs:1252 同款收尾——不经 View 消息面，宿主数据叶纪律）。
// - 订阅目标（Hello.app_name）：url query `app` 参数（与 token 同律），
//   缺省 "remote-frame"；musk 集成传 `&app=musk-canvas-embed` 与 web
//   消费面（canvas_embed_web.ts）同源对齐。
//
// web 轨不消费本模块（vue 臂 = 透传 div 占位，宿主前端接管）。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::ui::desktop_protocol::message::{FrameMsg, HandshakeMsg, ProtocolMsg};
use crate::ui::desktop_protocol::transport::Transport;
use crate::ui::desktop_protocol::PROTOCOL_VERSION;

/// 重连总预算（毫秒）——ReconnectPolicy 语义（client_runtime.rs:427）。
pub const RECONNECT_BUDGET_MS: u32 = 30_000;
/// 重连重试间隔（毫秒）。
pub const RECONNECT_INTERVAL_MS: u32 = 50;
/// 首连超时（毫秒；ws::connect 内部 20ms 循环重试至此限）。
pub const CONNECT_TIMEOUT_MS: u32 = 3_000;
/// attach 租约（毫秒）：渲染臂 note 续租；落后此时长 = url 退场 → 断连
/// 清理（5 个 ServiceTick 拍的容忍窗——元件瞬间条件渲染不抖断）。
pub const ATTACH_LEASE_MS: u32 = 2_000;
/// Hello 订阅名缺省值（url query `app` 覆盖）。
pub const DEFAULT_APP_NAME: &str = "remote-frame";

/// 客户端相位（占位渲染/诊断消费）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    /// 首连中（connect 线程在跑）。
    Connecting,
    /// 已连待兑现（Hello 已发，等 Welcome）。
    Subscribing,
    /// 订阅兑现（帧流在途）。
    Live,
    /// 断连重连预算中。
    Reconnecting,
    /// 预算尽（终态——占位诊断；url 退场后重入走新 Client）。
    Exhausted,
}

/// 订阅兑现载荷（Welcome 提取——输入换算的远端逻辑域基）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiveInfo {
    pub wid: u64,
    /// 远端逻辑域宽（Welcome.rect.w）。
    pub w: f32,
    /// 远端逻辑域高（Welcome.rect.h）。
    pub h: f32,
}

/// 渲染臂只读快照（每 present 一取；Arc 帧共享，无深拷贝）。
#[derive(Debug, Clone)]
pub struct RemoteFrameSnapshot {
    pub phase: Phase,
    pub live: Option<LiveInfo>,
    pub frame: Option<Arc<crate::ui::desktop_protocol::message::DisplayList>>,
}

struct Client {
    /// 当前端点（None = 连接中/重连排队/预算尽）。
    end: Option<Box<dyn Transport + Send>>,
    /// connect 线程结果槽（Connecting 态 Some；线程写终值后自然退出）。
    connect_slot: Option<
        Arc<
            Mutex<
                Option<
                    Result<
                        Box<dyn Transport + Send>,
                        crate::ui::desktop_protocol::transport::TransportError,
                    >,
                >,
            >,
        >,
    >,
    phase: Phase,
    live: Option<LiveInfo>,
    latest: Option<Arc<crate::ui::desktop_protocol::message::DisplayList>>,
    /// 重连预算终界（首次断连时刻 + BUDGET；连接态 None）。
    budget_deadline: Option<Instant>,
    /// 下次重拨时刻（Reconnecting 态节流）。
    next_dial: Option<Instant>,
    /// Hello 订阅名（url query `app` 提取）。
    app_name: String,
    last_seen: Instant,
}

impl Client {
    fn new(app_name: String) -> Self {
        Self {
            end: None,
            connect_slot: None,
            phase: Phase::Connecting,
            live: None,
            latest: None,
            budget_deadline: None,
            next_dial: None,
            app_name,
            last_seen: Instant::now(),
        }
    }

    /// 发 Hello 订阅（幂等——bridge 按 app_name 路由，同 web 消费面）。
    fn send_hello(&mut self) {
        if let Some(end) = self.end.as_mut() {
            let hello = ProtocolMsg::Handshake(HandshakeMsg::Hello {
                version: PROTOCOL_VERSION,
                app_name: self.app_name.clone(),
                title: String::new(),
                icon: None,
                width: 0.0,
                height: 0.0,
                fonts: vec![],
                surfaces: vec![],
            });
            let _ = end.send(&hello);
        }
    }

    fn snapshot(&self) -> RemoteFrameSnapshot {
        RemoteFrameSnapshot {
            phase: self.phase.clone(),
            live: self.live,
            frame: self.latest.clone(),
        }
    }
}

static REGISTRY: std::sync::LazyLock<Mutex<HashMap<String, Client>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// 测试隔离：全局 atomic 分配唯一 ns（并行测试各持不同值），存
/// thread-local 供 namespaced() 键加工；生产线程恒 0 = 原样 url 键。
static TEST_NS_ALLOC: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
thread_local! {
    static TEST_NS_CELL: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// url query 的 `app` 参数提取（缺省 DEFAULT_APP_NAME；`?token=..&app=..`）。
fn app_name_of(url: &str) -> String {
    let Some(query) = url.split_once('?').map(|(_, q)| q) else {
        return DEFAULT_APP_NAME.to_string();
    };
    for pair in query.split('&') {
        if let Some(v) = pair.strip_prefix("app=") {
            if !v.is_empty() {
                return v.to_string();
            }
        }
    }
    DEFAULT_APP_NAME.to_string()
}

fn namespaced(url: &str) -> String {
    let ns = TEST_NS_CELL.with(|c| c.get());
    if ns > 0 {
        format!("__t{ns}__{url}")
    } else {
        url.to_string()
    }
}

fn key_strip_ns(key: &str) -> String {
    // 测试命名空间剥离（__tN__ 前缀不入真连接 url）。
    if let Some(rest) = key.strip_prefix("__t") {
        if let Some(pos) = rest.find("__") {
            return rest[pos + 2..].to_string();
        }
    }
    key.to_string()
}

/// 渲染臂挂点：url 在场（续租；缺失则起首连线程）。返回只读快照。
pub fn note(url: &str) -> RemoteFrameSnapshot {
    let key = namespaced(url);
    let mut reg = REGISTRY.lock().unwrap();
    let client = reg
        .entry(key)
        .or_insert_with(|| Client::new(app_name_of(url)));
    client.last_seen = Instant::now();
    if client.phase == Phase::Connecting && client.connect_slot.is_none() && client.end.is_none() {
        client.connect_slot = Some(spawn_connect(url.to_string()));
    }
    client.snapshot()
}

/// 注册表非空（有 url 在租——standalone 模式的 ServiceTick 订阅拉起
/// 条件；url 退场租约清空后自动回落，稳态零成本）。
pub fn has_live_urls() -> bool {
    !REGISTRY.lock().unwrap().is_empty()
}

/// 只读快照（不续租——诊断/测试面）。
pub fn snapshot(url: &str) -> RemoteFrameSnapshot {
    let key = namespaced(url);
    let reg = REGISTRY.lock().unwrap();
    reg.get(&key)
        .map(|c| c.snapshot())
        .unwrap_or(RemoteFrameSnapshot {
            phase: Phase::Connecting,
            live: None,
            frame: None,
        })
}

/// 后台连接线程（ws::connect 阻塞循环的渲染线程隔离；写完结果自然退出）。
fn spawn_connect(
    url: String,
) -> Arc<
    Mutex<
        Option<
            Result<
                Box<dyn Transport + Send>,
                crate::ui::desktop_protocol::transport::TransportError,
            >,
        >,
    >,
> {
    let slot: Arc<
        Mutex<
            Option<
                Result<
                    Box<dyn Transport + Send>,
                    crate::ui::desktop_protocol::transport::TransportError,
                >,
            >,
        >,
    > = Arc::new(Mutex::new(None));
    let slot_writer = Arc::clone(&slot);
    std::thread::Builder::new()
        .name("remote-frame-dial".into())
        .spawn(move || {
            let result =
                crate::ui::desktop_protocol::transport::ws::connect(&url, CONNECT_TIMEOUT_MS);
            *slot_writer.lock().unwrap() = Some(result);
        })
        .expect("remote-frame dial thread");
    slot
}

/// ServiceTick 泵（非阻塞；renderer.rs pump_remote_mirrors 之后同拍）。
pub fn pump() {
    let now = Instant::now();
    let mut reg = REGISTRY.lock().unwrap();
    // ① 租约清理：url 退场 → Client 整体 Drop（WsEnd Drop = shutdown +
    // join 读线程；迟到帧随对象丢弃——身份防串）。
    let expired: Vec<String> = reg
        .iter()
        .filter(|(_, c)| {
            now.duration_since(c.last_seen) > Duration::from_millis(ATTACH_LEASE_MS as u64)
        })
        .map(|(k, _)| k.clone())
        .collect();
    for key in expired {
        reg.remove(&key);
    }
    let urls: Vec<String> = reg.keys().cloned().collect();
    for key in urls {
        let Some(client) = reg.get_mut(&key) else {
            continue;
        };
        // ② connect 结果收编（首连 Connecting 与重拨 Reconnecting 共路——
        // 拨号线程只写结果槽，收编统一在此；slot 空档 = 线程在跑）。
        if matches!(client.phase, Phase::Connecting | Phase::Reconnecting) {
            if let Some(slot) = client.connect_slot.as_ref() {
                let taken = slot.lock().unwrap().take();
                match taken {
                    Some(Ok(end)) => {
                        client.connect_slot = None;
                        client.end = Some(end);
                        client.phase = Phase::Subscribing;
                        client.send_hello();
                    }
                    Some(Err(_)) => {
                        // 拨号失败：保持/进入重连节流（预算从首次断连起算）。
                        client.connect_slot = None;
                        client.phase = Phase::Reconnecting;
                        client.budget_deadline.get_or_insert_with(|| {
                            now + Duration::from_millis(RECONNECT_BUDGET_MS as u64)
                        });
                        client.next_dial =
                            Some(now + Duration::from_millis(RECONNECT_INTERVAL_MS as u64));
                    }
                    None => {} // 线程仍在跑（CONNECT_TIMEOUT 内会出结果）。
                }
            }
        }
        // ③ 重连拨号节流（Reconnecting 态）。
        if client.phase == Phase::Reconnecting {
            let deadline = client.budget_deadline.unwrap_or(now);
            if now >= deadline {
                client.phase = Phase::Exhausted;
                client.next_dial = None;
                continue;
            }
            let due = client.next_dial.map(|t| now >= t).unwrap_or(true);
            if due && client.connect_slot.is_none() {
                let url = key_strip_ns(&key);
                client.connect_slot = Some(spawn_connect(url));
                client.next_dial = Some(now + Duration::from_millis(RECONNECT_INTERVAL_MS as u64));
            }
        }
        // ④ 收件排空（Subscribing/Live 共路——Welcome 兑现 + 帧槽更新）。
        // 借用分离:先经端点收走全部消息,再回写 client（end 是 client 字段
        // 的 &mut,apply_msg 也要 &mut client）。
        let mut drained: Vec<ProtocolMsg> = Vec::new();
        let mut saw_eof = false;
        if let Some(end) = client.end.as_mut() {
            loop {
                match end.try_recv() {
                    Some(Ok(msg)) => drained.push(msg),
                    // 坏载荷丢弃该条不断链（pump_remote_mirrors 同律）。
                    Some(Err(_)) => continue,
                    None => break,
                }
            }
            saw_eof = end.is_eof();
        }
        if std::env::var("AUTO_RF_DEBUG").map_or(false, |v| v == "1") {
            eprintln!(
                "[rf] url={} phase={:?} live={:?} latest={:?} drained={}",
                key_strip_ns(&key),
                client.phase,
                client.live,
                client.latest.as_ref().map(|f| f.ops.len()),
                drained.len()
            );
        }
        for msg in drained {
            apply_msg(client, &msg);
        }
        if saw_eof && client.end.is_some() {
            // 断连：端点丢弃（Drop 线程回收），预算窗内重拨；latest 同步
            // 清除（残帧会让 painter 继续画旧帧——占位态语义）。
            client.end = None;
            client.live = None;
            client.latest = None;
            client.phase = Phase::Reconnecting;
            client
                .budget_deadline
                .get_or_insert_with(|| now + Duration::from_millis(RECONNECT_BUDGET_MS as u64));
            client.next_dial = Some(now);
        }
    }
}

fn apply_msg(client: &mut Client, msg: &ProtocolMsg) {
    match msg {
        ProtocolMsg::Handshake(HandshakeMsg::Welcome { wid, rect, .. }) => {
            client.live = Some(LiveInfo {
                wid: *wid,
                w: rect.w.max(1.0),
                h: rect.h.max(1.0),
            });
            client.phase = Phase::Live;
        }
        ProtocolMsg::Frame(FrameMsg::FrameReadyV2 { payload, .. }) => {
            crate::ui::desktop_protocol::message::rf_textdump("host", payload);
            client.latest = Some(Arc::new(payload.clone()));
        }
        // v1 FrameReady：降格端帧（老宿主）——remote-frame 只消费 v2
        // DisplayList（rqhost bridge v2 优先；v1 载荷形态不同，忽略）。
        ProtocolMsg::Frame(FrameMsg::FrameReady { .. }) => {}
        _ => {}
    }
}

/// 输入直发（canvas update 臂；ProtocolMsg::Input(InputMsg) 同 rqhost 收尾）。
pub fn send_input(url: &str, msg: &ProtocolMsg) {
    if std::env::var("AUTO_RF_DEBUG").map_or(false, |v| v == "1") {
        eprintln!("[rf-input] {} {:?}", key_strip_ns(url), msg);
    }
    let key = namespaced(url);
    let mut reg = REGISTRY.lock().unwrap();
    if let Some(client) = reg.get_mut(&key) {
        if let Some(end) = client.end.as_mut() {
            let _ = end.send(msg);
        }
    }
}

// ---------------------------------------------------------------------------
// canvas 画师：draw（最新帧等比 fit 直绘 / 占位）+ update（指针 → InputMsg）。
// ---------------------------------------------------------------------------

use iced::widget::canvas::{self, Event as CanvasEvent, Frame, Text};
use iced::{mouse, Rectangle};

pub struct RemoteFramePainter<M> {
    pub url: String,
    _message: std::marker::PhantomData<fn() -> M>,
}

impl<M> RemoteFramePainter<M> {
    pub fn new(url: String) -> Self {
        Self {
            url,
            _message: std::marker::PhantomData,
        }
    }

    /// bounds 局部像素 → 远端逻辑域（等比换算——canvas_embed_web.ts
    /// toLogical 同一算式语义；draw 的 fit 缩放因子互逆）。
    /// 输入换算与 draw 同律（等比 contain + 居中偏移逆映射）——修复
    /// F-RF-1：原拉伸映射在宽高比不等时与渲染错位（760 e2e 的 480x680
    /// 1:1 窗口掩盖；musk 1280x800 宿主实测点击错行，PLAN-102 P-embed
    /// 定罪）。local 在内容盒外（letterbox）→ 负/超界坐标原样下发，由
    /// 远端按盒外语义处理（与 draw 不裁形口径一致）。
    fn to_remote(local: (f32, f32), bounds: &Rectangle, live: &LiveInfo) -> (f32, f32) {
        let lw = bounds.width.max(1.0);
        let lh = bounds.height.max(1.0);
        let (fw, fh) = (live.w.max(1.0), live.h.max(1.0));
        let scale = (lw / fw).min(lh / fh);
        if scale <= 0.0 {
            return local;
        }
        let dx = (lw - fw * scale) * 0.5;
        let dy = (lh - fh * scale) * 0.5;
        ((local.0 - dx) / scale, (local.1 - dy) / scale)
    }
}

impl<M> canvas::Program<M> for RemoteFramePainter<M> {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &CanvasEvent,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<M>> {
        use iced::mouse::Event as MouseEvent;
        // 远端未兑现（无 Welcome）时无输入语义——不捕获（事件冒泡宿主）。
        let snap = snapshot(&self.url);
        let Some(live) = snap.live else {
            return None;
        };
        let Some(local) = cursor.position_in(bounds) else {
            return None;
        };
        let (x, y) = Self::to_remote((local.x, local.y), &bounds, &live);
        if std::env::var("AUTO_RF_DEBUG").map_or(false, |v| v == "1") {
            eprintln!(
                "[rf-upd] local=({:.1},{:.1}) bounds=({:.1},{:.1} {:.1}x{:.1}) live=({:.1},{:.1}) remote=({:.1},{:.1})",
                local.x, local.y, bounds.x, bounds.y, bounds.width, bounds.height, live.w, live.h, x, y
            );
        }
        let wire = |input: crate::ui::desktop_protocol::message::InputMsg| {
            send_input(&self.url, &ProtocolMsg::Input(input));
        };
        match event {
            CanvasEvent::Mouse(MouseEvent::CursorMoved { .. }) => {
                wire(
                    crate::ui::desktop_protocol::message::InputMsg::PointerMoved {
                        wid: live.wid,
                        x,
                        y,
                    },
                );
            }
            CanvasEvent::Mouse(MouseEvent::ButtonPressed(button)) => {
                wire(
                    crate::ui::desktop_protocol::message::InputMsg::PointerPressed {
                        wid: live.wid,
                        button: wire_button(*button),
                        x,
                        y,
                        modifiers: 0,
                    },
                );
            }
            CanvasEvent::Mouse(MouseEvent::ButtonReleased(button)) => {
                wire(
                    crate::ui::desktop_protocol::message::InputMsg::PointerReleased {
                        wid: live.wid,
                        button: wire_button(*button),
                        x,
                        y,
                        modifiers: 0,
                    },
                );
            }
            _ => {}
        }
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let snap = note(&self.url);
        let mut frame = Frame::new(renderer, bounds.size());
        match (&snap.frame, snap.live) {
            (Some(list), live) => {
                // 有帧：等比 fit + 居中（与输入换算同一比例因子——DisplayList
                // 逻辑域重放不裁形；Welcome 缺席时按 1:1）。
                let lw = bounds.width.max(1.0);
                let lh = bounds.height.max(1.0);
                let (fw, fh) = live
                    .map(|l| (l.w.max(1.0), l.h.max(1.0)))
                    .unwrap_or((lw, lh));
                let scale = (lw / fw).min(lh / fh);
                let dx = (lw - fw * scale) * 0.5;
                let dy = (lh - fh * scale) * 0.5;
                frame.fill_rectangle(
                    iced::Point::ORIGIN,
                    bounds.size(),
                    iced::Color::from_rgb(0.12, 0.12, 0.14),
                );
                frame.translate(iced::Vector::new(dx, dy));
                frame.scale_nonuniform(iced::Vector::new(scale, scale));
                if let Some(clear) = list.clear {
                    frame.fill_rectangle(
                        iced::Point::ORIGIN,
                        iced::Size::new(fw, fh),
                        crate::ui::iced::broker_surface::to_color(clear),
                    );
                }
                crate::ui::iced::broker_surface::paint_ops_v2(&mut frame, &list.ops);
            }
            _ => {
                // 无帧占位：灰底 + 相位诊断文本（AC-02 占位态可视面）。
                frame.fill_rectangle(
                    iced::Point::ORIGIN,
                    bounds.size(),
                    iced::Color::from_rgb(0.12, 0.12, 0.14),
                );
                let label = match snap.phase {
                    Phase::Connecting | Phase::Subscribing => "connecting…",
                    Phase::Reconnecting => "reconnecting…",
                    Phase::Exhausted => "connection lost",
                    Phase::Live => "waiting for first frame…",
                };
                frame.fill_text(Text {
                    content: label.to_string(),
                    position: iced::Point::new(bounds.width * 0.5, bounds.height * 0.5),
                    color: iced::Color::from_rgb(0.62, 0.64, 0.68),
                    size: 12.0.into(),
                    align_x: iced::widget::text::Alignment::Center,
                    align_y: iced::alignment::Vertical::Center,
                    ..Default::default()
                });
            }
        }
        vec![frame.into_geometry()]
    }
}

/// iced 按钮 → 协议线 MouseButton（rqhost wire_button 同映射）。
fn wire_button(button: mouse::Button) -> crate::ui::desktop_protocol::message::MouseButton {
    use crate::ui::desktop_protocol::message::MouseButton;
    match button {
        mouse::Button::Left => MouseButton::Left,
        mouse::Button::Right => MouseButton::Right,
        mouse::Button::Middle => MouseButton::Middle,
        _ => MouseButton::Left,
    }
}

// ---------------------------------------------------------------------------
// 测试面：mock Transport 注入（Arc 共享态——注入后测试仍可观察出站、
// 续投入站、驱动 EOF）+ 命名空间隔离（全局表不互踩）。
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use crate::ui::desktop_protocol::codec::CodecError;
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicBool, AtomicUsize};

    /// mock 共享态（测试持 Arc 与注册表端点共享）。
    pub struct MockShared {
        inbound: Mutex<VecDeque<Vec<u8>>>,
        outbound: Mutex<Vec<Vec<u8>>>,
        eof: AtomicBool,
        /// 端点 Drop 计数（AC-04 close 断言面）。
        pub drops: AtomicUsize,
    }

    impl MockShared {
        pub fn new() -> Arc<Self> {
            Arc::new(Self {
                inbound: Mutex::new(VecDeque::new()),
                outbound: Mutex::new(Vec::new()),
                eof: AtomicBool::new(false),
                drops: AtomicUsize::new(0),
            })
        }

        /// 服务端投递一条消息（入站队列）。
        pub fn feed(&self, msg: &ProtocolMsg) {
            self.inbound.lock().unwrap().push_back(msg.encode());
        }

        /// 出站消息解码快照（Hello/Input 断言面）。
        pub fn sent(&self) -> Vec<ProtocolMsg> {
            self.outbound
                .lock()
                .unwrap()
                .iter()
                .filter_map(|b| ProtocolMsg::decode(b).ok())
                .collect()
        }

        pub fn set_eof(&self) {
            self.eof.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    struct MockTransport(Arc<MockShared>);

    impl Transport for MockTransport {
        fn send(
            &mut self,
            msg: &ProtocolMsg,
        ) -> Result<(), crate::ui::desktop_protocol::transport::TransportError> {
            self.0.outbound.lock().unwrap().push(msg.encode());
            Ok(())
        }

        fn write_raw(
            &self,
            bytes: &[u8],
        ) -> Result<(), crate::ui::desktop_protocol::transport::TransportError> {
            self.0.outbound.lock().unwrap().push(bytes.to_vec());
            Ok(())
        }

        fn try_recv(&mut self) -> Option<Result<ProtocolMsg, CodecError>> {
            self.0
                .inbound
                .lock()
                .unwrap()
                .pop_front()
                .map(|b| ProtocolMsg::decode(&b))
        }

        fn pending(&self) -> usize {
            self.0.inbound.lock().unwrap().len()
        }

        fn is_eof(&self) -> bool {
            self.0.eof.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    impl Drop for MockTransport {
        fn drop(&mut self) {
            self.0
                .drops
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }

    /// 测试注入：mock 端点直入注册表（Subscribing 态，Hello 随注入发出）。
    /// 测试持共享态全程可观察。
    pub fn inject(url: &str, shared: Arc<MockShared>) {
        let key = namespaced(url);
        let mut reg = REGISTRY.lock().unwrap();
        let mut client = Client::new(app_name_of(url));
        client.end = Some(Box::new(MockTransport(shared)));
        client.phase = Phase::Subscribing;
        client.send_hello();
        reg.insert(key, client);
    }

    /// 命名空间隔离：本线程 bump ns（thread-local——并行测试互不干扰），
    /// 条目入 __tN__ 空间；还原函数只清本命名空间条目。
    pub fn isolate() -> impl FnOnce() {
        let ns = TEST_NS_ALLOC.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        TEST_NS_CELL.with(|c| c.set(ns));
        let prefix = format!("__t{ns}__");
        move || {
            let mut reg = REGISTRY.lock().unwrap();
            reg.retain(|k, _| !k.starts_with(&prefix));
        }
    }

    /// 直接操纵注册表内 client 的 last_seen（租约时间旅行）。
    pub fn age_lease(url: &str) {
        let mut reg = REGISTRY.lock().unwrap();
        if let Some(c) = reg.get_mut(&namespaced(url)) {
            c.last_seen = Instant::now() - Duration::from_millis((ATTACH_LEASE_MS + 100) as u64);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    use crate::ui::desktop_protocol::message::{DisplayList, InputMsg, MouseButton, WRect};

    /// 注册表族测试串行锁——pump() 遍历全局表，并行测试的 pump 会交叉
    /// 消费彼此的 inbox（Welcome 抢跑相位断言）；持锁到测试结束。
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    /// F-RF-1：输入换算与 draw 同律（contain + 居中逆映射）——拉伸映射在
    /// 宽高比不等时错位（760 e2e 1:1 窗口掩盖；musk 1280x800 定罪）。
    #[test]
    fn to_remote_mirrors_draw_contain_fit() {
        // 宿主盒 397x735（musk 实测形态），远端 480x680 → scale=0.821、
        // dy=(735-558)/2=88.4。点 (41.5, 131.5) → 远端 (50.5, 54.7)
        //（拉伸映射得 (50.2, 121.6)——错位复现锚）。
        let bounds = Rectangle::new(iced::Point::ORIGIN, iced::Size::new(397.0, 735.0));
        let live = LiveInfo {
            wid: 2,
            w: 480.0,
            h: 680.0,
        };
        let (rx, ry) = RemoteFramePainter::<()>::to_remote((41.5, 131.5), &bounds, &live);
        assert!((rx - 50.2).abs() < 0.2, "x={rx}");
        assert!((ry - 54.7).abs() < 0.2, "y={ry}（拉伸映射得 121.6——错位）");
        // 1:1 恒等（760 e2e 形态）不变。
        let sq = Rectangle::new(iced::Point::ORIGIN, iced::Size::new(480.0, 680.0));
        let (ix, iy) = RemoteFramePainter::<()>::to_remote((240.0, 340.0), &sq, &live);
        assert!(
            (ix - 240.0).abs() < 0.01 && (iy - 340.0).abs() < 0.01,
            "1:1 恒等破坏"
        );
    }

    fn welcome(wid: u64, w: f32, h: f32) -> ProtocolMsg {
        ProtocolMsg::Handshake(HandshakeMsg::Welcome {
            app_id: 1,
            wid,
            surface: 0,
            rect: WRect {
                x: 0.0,
                y: 0.0,
                w,
                h,
            },
            frame_mode: crate::ui::desktop_protocol::message::FrameMode::Commands,
            extra_surfaces: vec![],
        })
    }

    fn frame_v2(frame_id: u64, blue: u8) -> ProtocolMsg {
        ProtocolMsg::Frame(FrameMsg::FrameReadyV2 {
            wid: 7,
            frame_id,
            slot: 0,
            damage: None,
            revision: frame_id,
            payload: DisplayList {
                clear: Some(crate::ui::desktop_protocol::message::Rgba8 {
                    r: 20,
                    g: 40,
                    b: blue,
                    a: 255,
                }),
                ops: vec![],
            },
        })
    }

    /// 生命周期主路：注入（Hello 出站）→ Welcome 兑现 → 帧槽更新 →
    /// url 退场 = 租约过期 → Client Drop（mock drops 计数 = close 断言）。
    #[test]
    fn lifecycle_welcome_frame_and_lease_cleanup() {
        let _serial = TEST_LOCK.lock().unwrap();
        let restore = isolate();
        let url = "ws://127.0.0.1:19860/?token=t&app=fixture-app";
        let shared = MockShared::new();
        shared.feed(&welcome(7, 480.0, 320.0));
        shared.feed(&frame_v2(1, 90));
        inject(url, shared.clone());

        let snap = note(url);
        assert_eq!(snap.phase, Phase::Subscribing, "注入后应为 Subscribing");
        // Hello 出站：订阅名 = url query app 参数。
        let sent = shared.sent();
        assert!(
            sent.iter().any(|m| matches!(
                m,
                ProtocolMsg::Handshake(HandshakeMsg::Hello { app_name, .. })
                    if app_name == "fixture-app"
            )),
            "Hello 订阅名应取 url query app=fixture-app; got {sent:?}"
        );

        pump();
        let snap = snapshot(url);
        assert_eq!(snap.phase, Phase::Live, "Welcome 后应 Live");
        let live = snap.live.expect("live 载荷");
        assert_eq!((live.wid, live.w, live.h), (7, 480.0, 320.0));
        let frame = snap.frame.expect("帧应入槽");
        assert_eq!(frame.clear.map(|c| c.b), Some(90), "帧载荷 = 预投 v2 帧");

        // url 退场：租约时间旅行 → pump 清理 → Transport Drop（close）。
        assert_eq!(shared.drops.load(std::sync::atomic::Ordering::SeqCst), 0);
        age_lease(url);
        pump();
        assert_eq!(
            shared.drops.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "租约到期应 Drop 端点（AC-04 close 断言）"
        );
        assert!(
            REGISTRY
                .lock()
                .unwrap()
                .keys()
                .all(|k| !k.contains("19860")),
            "退场 url 应出册"
        );
        restore();
    }

    /// 输入直发：send_input → mock 出站收到 InputMsg（PointerPressed）。
    #[test]
    fn input_forwarding_to_wire() {
        let _serial = TEST_LOCK.lock().unwrap();
        let restore = isolate();
        let url = "ws://127.0.0.1:19862/?token=t";
        let shared = MockShared::new();
        shared.feed(&welcome(9, 200.0, 100.0));
        inject(url, shared.clone());
        pump();

        send_input(
            url,
            &ProtocolMsg::Input(InputMsg::PointerPressed {
                wid: 9,
                button: MouseButton::Left,
                x: 42.0,
                y: 24.0,
                modifiers: 0,
            }),
        );
        let sent = shared.sent();
        assert!(
            sent.iter().any(|m| matches!(
                m,
                ProtocolMsg::Input(InputMsg::PointerPressed { wid, x, y, .. })
                    if *wid == 9 && *x == 42.0 && *y == 24.0
            )),
            "PointerPressed 应达出站; got {sent:?}"
        );
        assert_eq!(snapshot(url).phase, Phase::Live, "输入发送不改变相位");
        restore();
    }

    /// 重连状态机：EOF → Reconnecting（预算窗内）→ 预算尽 → Exhausted
    /// 终态。
    #[test]
    fn reconnect_budget_then_exhausted() {
        let _serial = TEST_LOCK.lock().unwrap();
        let restore = isolate();
        let url = "ws://127.0.0.1:19863/?token=t";
        let shared = MockShared::new();
        shared.feed(&welcome(7, 100.0, 100.0));
        inject(url, shared.clone());
        pump();
        assert_eq!(snapshot(url).phase, Phase::Live);

        // EOF：端点自报 → pump 转 Reconnecting（端点 Drop + 重拨排队）。
        shared.set_eof();
        pump();
        assert_eq!(snapshot(url).phase, Phase::Reconnecting, "EOF 后应进重连");
        assert_eq!(
            shared.drops.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "死端点应被 Drop"
        );

        // 预算耗尽（deadline 时间旅行）→ Exhausted 终态。
        {
            let mut reg = REGISTRY.lock().unwrap();
            reg.get_mut(&namespaced(url)).unwrap().budget_deadline =
                Some(Instant::now() - Duration::from_millis(1));
        }
        pump();
        assert_eq!(snapshot(url).phase, Phase::Exhausted, "预算尽应终态占位");
        restore();
    }

    /// 重连收编回归（e2e 2026-10-11 死锁修复）：Reconnecting 态拨号线程
    /// 的结果必须被 pump 收编（曾只在 Connecting 分支收编——重拨永不成
    /// 功）。模拟：EOF → Reconnecting → 注入"已完成"的结果槽 → pump
    /// 收编转 Subscribing。
    #[test]
    fn reconnect_dial_result_is_harvested() {
        let _serial = TEST_LOCK.lock().unwrap();
        let restore = isolate();
        let url = "ws://127.0.0.1:19864/?token=t";
        let shared = MockShared::new();
        shared.feed(&welcome(5, 100.0, 100.0));
        inject(url, shared.clone());
        pump();
        assert_eq!(snapshot(url).phase, Phase::Live);

        // 断连进 Reconnecting，手动装一个"已失败"的拨号结果 → pump 收编
        // 后应保持 Reconnecting 且槽清空（下一拍可再拨）。
        shared.set_eof();
        pump();
        assert_eq!(snapshot(url).phase, Phase::Reconnecting);
        // EOF 转换在 pump ④，拨号 spawn 在下一拍 ③——再 pump 一次后在途。
        pump();
        {
            let mut reg = REGISTRY.lock().unwrap();
            let c = reg.get_mut(&namespaced(url)).unwrap();
            assert!(c.connect_slot.is_some(), "Reconnecting 应有在途拨号");
        }
        // 收编路径已被 ② 覆盖（matches! Connecting | Reconnecting）——结果
        // 由真线程写；等一拍让线程完成（CONNECT_TIMEOUT 内必出）。
        std::thread::sleep(Duration::from_millis(200));
        pump();
        // （拨号目标是真端口 19864——大概率失败 → 仍 Reconnecting；关键
        // 断言：槽被消费清空、进程不死锁、预算推进。）
        let reg = REGISTRY.lock().unwrap();
        let c = reg.get(&namespaced(url)).expect("client 仍在册（预算内）");
        assert_eq!(c.phase, Phase::Reconnecting, "拨号失败应停留预算内重连");
        drop(reg);
        restore();
    }

    /// url query app 提取（订阅名契约——musk 集成 &app=musk-canvas-embed）。
    #[test]
    fn app_name_from_url_query() {
        assert_eq!(
            app_name_of("ws://h:1/?token=t&app=musk-canvas-embed"),
            "musk-canvas-embed"
        );
        assert_eq!(app_name_of("ws://h:1/?token=t"), DEFAULT_APP_NAME);
        assert_eq!(app_name_of("ws://h:1"), DEFAULT_APP_NAME);
    }
}

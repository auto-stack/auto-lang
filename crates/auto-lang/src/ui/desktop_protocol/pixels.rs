// Plan 500 步骤 4 —— Pixels 泵（independent 臂 child 侧）。
//
// T1 定案路径（设计 §1.3 Pixels 路径行）：child 自带 iced 运行时 +
// **隐藏窗**（app 尺寸）→ 状态变更/输入后重渲染 → `iced::window::
// screenshot` 整窗抓取（497 T1 已验证的唯一公开栅格化通道；物理像素 ×
// scale_factor）→ RGBA 写 shm 槽（`[u32 len][rgba]` 统一槽框架——载荷
// 解释随 `Welcome.frame_mode` 而定）→ [`FrameMsg::FrameReadyPixels`]
// 元数据过管道。
//
// 分层：
// - [`PixelsFrame`]：一帧 RGBA + 元数据（screenshot → 帧的纯转换）。
// - [`PixelsChild`]：child 侧桥——协议端点（握手/槽纪律/生命周期）+
//   shm 段 + 截图泵（`on_protocol` / `capture`）。渲染宿主（iced daemon）
//   持有它并从 update 周期驱动；单测以合成帧驱动同一路径（帧递增）。
// - 输入边界（v1.3）：协议 Input 触发重渲染 + 截图，但不做 handler
//   派发——命中区表来自 iced 布局属 Stage 5（D3 细化）；queue 臂的输入
//   闭环由 AppProjector 命中表承担（本计划验收口径）。

use std::sync::{Arc, Mutex, OnceLock};

use super::endpoint::{AppEndpoint, FrameSource};
use super::message::{DrawList, FrameMode, FrameMsg, HandshakeMsg, InputMsg, ProtocolMsg};
use super::shm::SharedFrameBuffer;
use super::transport::Transport;

/// 一帧像素（straight RGBA8 非预乘；来自 screenshot 或测试合成源）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelsFrame {
    pub rgba: Vec<u8>,
    pub w: u32,
    pub h: u32,
    pub stride: u32,
}

impl PixelsFrame {
    /// w×h 纯色测试帧（stride = w×4）。
    #[cfg(test)]
    pub fn solid(w: u32, h: u32, rgba: [u8; 4]) -> Self {
        let stride = w * 4;
        let mut bytes = Vec::with_capacity((stride * h) as usize);
        for _ in 0..w * h {
            bytes.extend_from_slice(&rgba);
        }
        Self { rgba: bytes, w, h, stride }
    }
}

/// 像素臂端点的会话垫片：内容渲染归 iced（宿主侧桥接），端点只用其
/// 协议状态机（握手/槽纪律/frame_id/Close 生命周期）——`render_frame`
/// 永不被调（帧生产走 [`PixelsChild::capture`] 的像素路径）。
#[derive(Debug, Default)]
pub struct PixelsNoopSource {
    rev: std::cell::Cell<u64>,
}

impl PixelsNoopSource {
    pub fn new() -> Self {
        Self { rev: std::cell::Cell::new(1) }
    }

    /// 内容版本推进（状态变更/快照注入时 +1）。
    pub fn bump(&self) {
        self.rev.set(self.rev.get() + 1);
    }
}

impl FrameSource for PixelsNoopSource {
    fn revision(&self) -> u64 {
        self.rev.get()
    }

    fn render_frame(&mut self) -> DrawList {
        DrawList::default() // 像素臂不产命令帧
    }

    fn on_input(&mut self, _input: &InputMsg) {}

    fn on_control(&mut self, _control: &super::message::ControlMsg) {}
}

/// independent 臂 child 桥（渲染宿主持有，update 周期驱动）。
pub struct PixelsChild {
    /// 到宿主的管道（读线程搬进 inbox、写侧阻塞写——Arc<Mutex> 供
    /// 订阅层轮询与 update 层发送共用）。
    pub transport: Arc<Mutex<Box<dyn Transport + Send>>>,
    pub endpoint: AppEndpoint<PixelsNoopSource>,
    /// BufferAlloc 开段后有效（槽尺寸 = 像素帧上限，见 [`open_shm`]）。
    pub shm: Option<SharedFrameBuffer>,
    /// 截图泵状态：true = 有未发起的抓取（去重防抖）。
    capture_pending: bool,
}

/// 像素帧 shm 槽尺寸：`w×h×4` + 4 字节 len 前缀（双槽同尺寸）。
pub fn pixels_slot_size(width: f32, height: f32) -> u32 {
    let w = width.max(1.0).ceil() as u32;
    let h = height.max(1.0).ceil() as u32;
    w * h * 4 + 4
}

impl PixelsChild {
    pub fn new(
        transport: Arc<Mutex<Box<dyn Transport + Send>>>,
        app_name: &str,
        title: &str,
        width: f32,
        height: f32,
    ) -> Self {
        Self {
            transport,
            endpoint: AppEndpoint::new(PixelsNoopSource::new(), app_name, title, width, height),
            shm: None,
            capture_pending: false,
        }
    }

    /// 发起握手（Hello 过管道）。返回 false = 管道已断。
    pub fn start(&mut self) -> bool {
        let Ok(hello) = self.endpoint.connect() else { return false };
        self.send(&hello)
    }

    /// 消息出口（update 层调用；发送失败 = 断连，渲染宿主收尾）。
    pub fn send(&mut self, msg: &ProtocolMsg) -> bool {
        self.transport.lock().map(|mut t| t.send(msg).is_ok()).unwrap_or(false)
    }

    /// 处理一条宿主消息：端点状态机 + 像素臂语义（BufferAlloc 开段 /
    /// StateSnapshot 写回真实组件由渲染宿主侧接线）。返回需回发的消息
    /// 与"是否需要发起截图"。
    pub fn on_protocol(
        &mut self,
        msg: ProtocolMsg,
        component: Option<&mut crate::ui::dynamic::DynamicComponent>,
    ) -> (Vec<ProtocolMsg>, bool) {
        // StateSnapshot：先逐字段写真实组件（像素臂的内容真源），再过
        // 端点状态机（其会话垫片不承载状态）。
        if let ProtocolMsg::Control(super::message::ControlMsg::StateSnapshot { payload, .. }) =
            &msg
        {
            if let Some(component) = component {
                if let Ok((revision, fields)) =
                    super::client_runtime::decode_state_snapshot(payload)
                {
                    for (field, value) in fields {
                        let _ = component.write_state(&field, value);
                    }
                    let _ = revision;
                }
            }
        }
        let replies = match self.endpoint.on_message(msg.clone()) {
            Ok(r) => r,
            Err(_) => return (Vec::new(), false),
        };
        let mut want_capture = false;
        match &msg {
            ProtocolMsg::Handshake(HandshakeMsg::Welcome { frame_mode: FrameMode::Pixels, .. }) => {
                // Pixels 模式确认：握手完成即出首帧。
                want_capture = true;
            }
            ProtocolMsg::Frame(FrameMsg::BufferAlloc { shm: Some(name), width, height, .. }) => {
                let slot_size = pixels_slot_size(*width, *height);
                match SharedFrameBuffer::open(name, 2, slot_size) {
                    Ok(segment) => {
                        self.shm = Some(segment);
                        want_capture = self.endpoint.state
                            == super::endpoint::AppState::Active;
                    }
                    Err(_) => return (replies, false),
                }
            }
            ProtocolMsg::Input(_) => {
                // v1.3 输入边界：无 handler 派发，触发重渲染 + 截图。
                if self.endpoint.state == super::endpoint::AppState::Active {
                    want_capture = true;
                }
            }
            ProtocolMsg::Control(super::message::ControlMsg::StateSnapshot { .. }) => {
                self.endpoint.session.bump();
                if self.endpoint.state == super::endpoint::AppState::Active {
                    want_capture = true;
                }
            }
            _ => {}
        }
        (replies, want_capture)
    }

    /// 渲染宿主发起截图前调用（去重：同轮只发一次 Task）。
    pub fn request_capture(&mut self) -> bool {
        if self.capture_pending || self.shm.is_none() {
            return false;
        }
        self.capture_pending = true;
        true
    }

    /// 截图回调 → 写 shm 槽 + FrameReadyPixels。返回 None = 无段/未握手。
    pub fn capture(&mut self, frame: PixelsFrame) -> Option<FrameMsg> {
        self.capture_pending = false;
        let shm = self.shm.as_ref()?;
        let msg = self.endpoint.produce_frame_pixels(
            shm,
            &frame.rgba,
            frame.w,
            frame.h,
            frame.stride,
        );
        msg.ok().and_then(|m| match m {
            ProtocolMsg::Frame(frame) => Some(frame),
            _ => None,
        })
    }

    /// 会话已 Detached（Close 生命周期走完）→ 渲染宿主应收尾退出。
    pub fn is_detached(&self) -> bool {
        self.endpoint.state == super::endpoint::AppState::Detached
    }
}

// ---------------------------------------------------------------------------
// 渲染宿主装配（boot 取桥 + 订阅轮询 + 入口）
// ---------------------------------------------------------------------------

/// boot 期取走的像素桥（`run_independent_child` 装填 → Standalone boot
/// 消费进 `session.pixels` 并发起 Hello）。MCP_ACTION_RX 同型全局。
static PIXELS_LAUNCH: OnceLock<Mutex<Option<PixelsChild>>> = OnceLock::new();

fn launch_slot() -> &'static Mutex<Option<PixelsChild>> {
    PIXELS_LAUNCH.get_or_init(|| Mutex::new(None))
}

/// 渲染宿主 boot 臂消费：取走像素桥（None = 非像素臂进程）。
pub fn take_launch() -> Option<PixelsChild> {
    launch_slot().lock().unwrap().take()
}

/// independent 臂 child 入口：真 iced 渲染宿主（run_session Standalone
/// 管线，窗口隐藏）+ 像素桥。阻塞至宿主 Close（Detached → iced::exit）
/// 或宿主窗全部关闭。cmd_autodesk 三态裁决（步骤 6）在 `independent`
/// 档调用本入口。
pub fn run_independent_child(
    transport: Box<dyn Transport + Send>,
    component: crate::ui::dynamic::DynamicComponent,
    app_name: &str,
    title: &str,
    width: f32,
    height: f32,
) -> Result<String, String> {
    let transport = Arc::new(Mutex::new(transport));
    let child = PixelsChild::new(Arc::clone(&transport), app_name, title, width, height);
    *launch_slot().lock().unwrap() = Some(child);
    let _ = PIXELS_POLL.set(Arc::clone(&transport));
    crate::ui::iced::run_dynamic_iced_pixels(component)
        .map_err(|e| format!("pixels child: {e}"))
}

/// 像素桥协议轮询订阅（MCP/native_dock 订阅同型：std 通道短轮询——
/// PipeEnd 读线程已把帧搬进 inbox，try_recv 零阻塞）。child 管道 EOF =
/// 流终止（订阅 diff 重订阅自愈，下一轮消息面自然重拉）。
pub fn pixels_protocol_subscription()
-> iced::Subscription<crate::ui::session::DesktopMessage> {
    use crate::ui::session::{DesktopEvent, DesktopMessage};

    struct PixelsProtocolRecipe;

    impl std::hash::Hash for PixelsProtocolRecipe {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            "auto-lang-pixels-protocol".hash(state);
        }
    }

    impl iced_futures::subscription::Recipe for PixelsProtocolRecipe {
        type Output = DesktopMessage;

        fn hash(&self, state: &mut iced_futures::subscription::Hasher) {
            std::hash::Hash::hash(self, state);
        }

        fn stream(
            self: Box<Self>,
            _input: iced_futures::subscription::EventStream,
        ) -> iced_futures::BoxStream<Self::Output> {
            use iced_futures::futures::stream::{StreamExt, unfold};
            unfold((), |()| async move {
                loop {
                    // 管道 inbox 探测（PipeEnd 读线程已搬帧，try_recv 零阻塞）；
                    // 空拍隔 5ms 重试，消息到达即转 update 事件面。
                    let polled = poll_transport();
                    match polled {
                        Some(Ok(msg)) => {
                            return Some((
                                DesktopMessage::Desktop(DesktopEvent::PixelsProtocol(Box::new(
                                    msg,
                                ))),
                                (),
                            ));
                        }
                        Some(Err(_codec)) => continue,
                        None => {
                            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                        }
                    }
                }
            })
            .boxed()
        }
    }

    iced_futures::subscription::from_recipe(PixelsProtocolRecipe)
}

/// 订阅层轮询的管道句柄（`run_independent_child` 装填；child 端点在
/// session 内，管道 Arc 两处共享——读 inbox 与 update 层发送互不阻塞）。
static PIXELS_POLL: OnceLock<Arc<Mutex<Box<dyn Transport + Send>>>> = OnceLock::new();

fn poll_transport() -> Option<Result<ProtocolMsg, super::codec::CodecError>> {
    PIXELS_POLL
        .get()?
        .lock()
        .ok()
        .and_then(|mut guard| guard.try_recv())
}


// ---------------------------------------------------------------------------
// 测试：合成帧驱动的泵全循环（单 client 帧递增——步骤 4 验收）
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::desktop_protocol::message::{MouseButton, WRect};
    use crate::ui::desktop_protocol::shm::SharedFrameBuffer;
    use crate::ui::desktop_protocol::transport;

    /// 合成帧泵：握手 → Welcome(Pixels) → BufferAlloc → 截图帧 ×3 →
    /// frame_id/槽轮转/槽字节/revision 全链断言。
    #[test]
    fn pixels_child_frame_increment() {
        let pipe = format!("autodesk-pixels-{}", std::process::id());
        let listener = transport::listen(&pipe).expect("listen");
        let end = transport::connect(&pipe, 2000).expect("connect");
        let transport = Arc::new(Mutex::new(end));
        let mut child = PixelsChild::new(
            Arc::clone(&transport),
            "hello",
            "hello",
            64.0,
            32.0,
        );
        let mut host_end = listener.wait_connect().expect("server");

        // 握手：child Hello → 宿主 Welcome(Pixels) + BufferAlloc。
        assert!(child.start());
        let hello = host_end.recv_wait(2000).expect("hello").expect("decode");
        let ProtocolMsg::Handshake(HandshakeMsg::Hello { app_name, .. }) = &hello else {
            panic!("期待 Hello");
        };
        assert_eq!(app_name, "hello");

        // 宿主侧段（槽尺寸 = 像素上限；双槽）。
        let shm_name = format!("autodesk-shm-pixels-{}", std::process::id());
        let host_shm =
            SharedFrameBuffer::create(&shm_name, 2, pixels_slot_size(64.0, 32.0)).expect("shm");
        host_end
            .send(&ProtocolMsg::Handshake(HandshakeMsg::Welcome {
                app_id: 1,
                wid: 7,
                surface: 42,
                rect: WRect::new(0.0, 0.0, 64.0, 32.0),
                frame_mode: FrameMode::Pixels,
            }))
            .unwrap();
        host_end
            .send(&ProtocolMsg::Frame(FrameMsg::BufferAlloc {
                surface: 42,
                slots:  2,
                width:  64.0,
                height: 32.0,
                shm:    Some(shm_name.clone()),
            }))
            .unwrap();

        // child 消费 Welcome + BufferAlloc → 开段 + 首帧截图请求
        //（Welcome 先到时段未开——request_capture 守卫拦下，等 BufferAlloc）。
        let mut frames = Vec::new();
        for _ in 0..100 {
            let mut guard = transport.lock().unwrap();
            let got = guard.recv_wait(20);
            drop(guard);
            let Some(Ok(msg)) = got else { continue };
            let (replies, want_capture) = child.on_protocol(msg, None);
            for r in replies {
                host_end.send(&r).unwrap();
            }
            if want_capture && child.shm.is_some() {
                assert!(child.request_capture(), "段在册的截图请求不被去重");
                let f1 = PixelsFrame::solid(64, 32, [200, 100, 50, 255]);
                let m = child.capture(f1).expect("首帧");
                frames.push(m);
            }
            if child.shm.is_some() && !frames.is_empty() {
                break;
            }
        }
        assert!(child.shm.is_some(), "BufferAlloc 开段");
        // Ready 回发已过管道（端点握手纪律）。
        assert_eq!(
            child.endpoint.state,
            crate::ui::desktop_protocol::endpoint::AppState::Active
        );

        // 帧递增 ×2：frame_id 单调、槽交替、宿主从 shm 读到字节。
        let f2 = PixelsFrame::solid(64, 32, [10, 20, 30, 255]);
        let f3 = PixelsFrame::solid(64, 32, [1, 2, 3, 255]);
        for (i, frame) in [f2, f3].into_iter().enumerate() {
            child.request_capture();
            let msg = child.capture(frame).expect(&format!("帧 {}", i + 2));
            frames.push(msg);
        }

        let ids: Vec<u64> = frames.iter().map(|m| frame_id_of(m)).collect();
        assert_eq!(ids, vec![1, 2, 3], "frame_id 单调递增");
        let slots: Vec<u8> = frames.iter().map(|m| slot_of(m)).collect();
        assert_ne!(slots[0], slots[1], "槽轮转（双缓冲翻转）");
        assert_eq!(slots[0], slots[2], "两槽循环");

        // 元数据一致性 + 宿主读槽字节 = 帧内容（straight RGBA）。
        let last = frames.last().unwrap();
        let FrameMsg::FrameReadyPixels { w, h, stride, revision, .. } = last else {
            panic!();
        };
        assert_eq!((*w, *h, *stride), (64, 32, 256));
        assert_eq!(*revision, 1, "无状态变更 revision 不动");
        let slot = slot_of(last);
        let bytes = host_shm.read_slot(slot).expect("read");
        assert_eq!(bytes.len(), (64 * 32 * 4) as usize);
        assert_eq!(&bytes[..4], &[1, 2, 3, 255], "末帧槽内容 = f3");

        // 输入消息：无 handler 派发但触发截图（v1.3 输入边界）。
        let (replies, want) = child.on_protocol(
            ProtocolMsg::Input(InputMsg::PointerPressed {
                wid: 7,
                button: MouseButton::Left,
                x: 10.0,
                y: 10.0,
                modifiers: 0,
            }),
            None,
        );
        assert!(replies.is_empty());
        assert!(want, "输入触发截图");
        assert!(child.request_capture());
        let m4 = child.capture(PixelsFrame::solid(64, 32, [9, 9, 9, 255])).unwrap();
        assert_eq!(frame_id_of(&m4), 4);
    }

    fn frame_id_of(m: &FrameMsg) -> u64 {
        let FrameMsg::FrameReadyPixels { frame_id, .. } = m else { panic!("FrameReadyPixels") };
        *frame_id
    }

    fn slot_of(m: &FrameMsg) -> u8 {
        let FrameMsg::FrameReadyPixels { slot, .. } = m else { panic!("FrameReadyPixels") };
        *slot
    }
}

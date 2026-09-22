// Plan 386 Stage 1 —— 桌面协议双端状态机。
//
// 本文件是纯协议层：不依赖 iced / 会话对象，只对 [`ProtocolMsg`] 做
// 方向与时序校验。宿主侧把 [`HostAction`] 落到 462 会话对象的适配在
// `host`（[`super::ProtocolHost`]）；app 侧的"会话"由 [`FrameSource`]
// 抽象——Stage 1 = 进程内 `DynamicComponent` 适配，Stage 2 = 独立 exe
// 的渲染循环，状态机零改动。
//
// 状态机（箭头只允许图内迁移，其余一律 [`ProtocolError::WrongState`]）：
//
// ```text
// App  : Detached --connect()--> Handshaking --Welcome(+Ready)--> Active
//        Active --Control::Close--> Closing --ExitRequest--> Detached
//        Active --send_exit()-->    Closing
// Host : Listening --Hello--> (ResolveAndAttach) --activate()--> Active
//        Active --ExitRequest--> (ReclaimWindow) --> Listening
// ```

use super::codec::CodecError;
use super::transport::TransportError;
use super::message::{
    ControlMsg, DrawList, FrameMode, HandshakeMsg, InputMsg, ObserveMsg, ProtocolMsg, WRect,
};

/// 协议状态机错误（方向错 / 时序错 / 非 Active 操作）。
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolError {
    /// 当前状态不接受该消息（携带状态名与消息描述）。
    WrongState { state: &'static str, msg: &'static str },
    /// 载荷编解码失败（透传）。
    Codec(CodecError),
    /// 需要 Active 状态的操作（如产帧）。
    NotActive,
    /// 握手版本协商失败（携带 app 上报的版本）。
    VersionMismatch(u16),
    /// 孵化材料解析失败（app_name 未知 / 编译失败——携带宿主侧原因）。
    ResolveFailed(String),
    /// 共享内存操作失败（S9）。
    Shm(TransportError),
}

impl From<CodecError> for ProtocolError {
    fn from(e: CodecError) -> Self {
        Self::Codec(e)
    }
}

// ---------------------------------------------------------------------------
// app 侧
// ---------------------------------------------------------------------------

/// App 端点状态（五通道公共时序）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    /// 未连接（`connect()` 未调用，或对端已回收）。
    Detached,
    /// Hello 已发出，等 Welcome。
    Handshaking,
    /// 握手完成，帧/输入/控制/观测全通道可用。
    Active,
    /// 收到 Close 或已发 ExitRequest，等宿主回收确认。
    Closing,
    /// L2"独立出去"：自管表面（自开 OS 窗/独立渲染循环），会话状态
    /// 不动；仅等 `request_attach()` 回归或进程退出。
    Standalone,
}

impl AppState {
    fn name(&self) -> &'static str {
        match self {
            Self::Detached => "Detached",
            Self::Handshaking => "Handshaking",
            Self::Active => "Active",
            Self::Closing => "Closing",
            Self::Standalone => "Standalone",
        }
    }
}

/// PLAN-034 T-05（D3）：app 侧位图上传请求（[`FrameSource::
/// drain_bitmap_uploads`] 排水面载荷）。RGBA straight 非预乘——
/// FrameReadyPixels 同口径。
#[derive(Debug, Clone)]
pub struct BitmapUpload {
    /// 位图标识（进程内唯一——组件侧经 [`bitmap_src`] 生成引用串）。
    pub id: String,
    pub w: u32,
    pub h: u32,
    /// 行字节距（紧排 = w×4；宽于 w×4 时宿主侧行重排）。
    pub stride: u32,
    pub rgba: Vec<u8>,
}

/// 位图局部 id（进程唯一化：pid 前缀——跨 app 撞名由构造侧消化，D3）。
pub fn bitmap_local_id(local: &str) -> String {
    format!("{}-{}", std::process::id(), local)
}

/// `bitmap://{pid}-{local}` 引用串（DrawOp Image src 词汇面——上传侧与
/// 引用侧单源同构，宿主键 = 同式直拼）。
pub fn bitmap_src(local: &str) -> String {
    format!("bitmap://{}", bitmap_local_id(local))
}

/// app 侧会话抽象：帧的产出 + 输入/控制的消费。
/// （Stage 1 的实现绑定 `DynamicComponent`，见 `host` 测试与 demo。）
pub trait FrameSource {
    /// 内容单调版本（每次可观测状态变化 +1；宿主缓存键，413 §7.3 同源）。
    fn revision(&self) -> u64;
    /// 产一帧载荷。
    fn render_frame(&mut self) -> DrawList;
    /// 输入事件注入（协议 → 会话事件循环）。
    fn on_input(&mut self, input: &InputMsg);
    /// 控制消息消费（焦点/resize 等生命周期语义）。
    fn on_control(&mut self, control: &ControlMsg);
    /// 周期拍（Plan 020 T-04）：泵循环每轮调用——解释态投影器无 tick 源
    /// （缺省空实现，行为零变化）；native 投影器实现为 interval 到期 →
    /// `component.on(tick_msg)` + revision 前进（`Component::tick_interval_ms`
    /// 配方，run_app_devtools 同源）。revision 变化由泵侧对账产帧。
    fn poll_tick(&mut self) {}

    /// PLAN-033 T-03③（D3）：`__desktop_cmd` 命令读走（泵在读走点消费
    /// 后上行 `ControlMsg::DesktopBus`）。缺省空——无命令面的会话零实现
    /// 负担；native 投影器经 [`crate::ui::Component::drain_desktop_commands`]
    /// 转发（VM 组件 c4 语义实现）。
    fn drain_desktop_commands(&mut self) -> Vec<String> {
        Vec::new()
    }

    /// PLAN-034 T-05（D3）：位图上传读走（`drain_desktop_commands` 同型
    /// 缝——033 先例）。缺省空——无位图生产面的会话零负担；canvas 快照
    /// / 合成生产者实现。泵在输入派发与周期拍后排水（drain 点同
    /// `drain_desktop_bus`）。
    fn drain_bitmap_uploads(&mut self) -> Vec<BitmapUpload> {
        Vec::new()
    }

    /// PLAN-683（remote 模式）：v2 产线帧。缺省 = v1 帧 lift（
    /// [`super::message::DisplayList::from_v1`]——语义保真直搬，legacy
    /// 生产者零改造）；headless 宿主覆写为 tiny_skia 记录层原生降格。
    fn render_frame_v2(&mut self) -> super::message::DisplayList {
        super::message::DisplayList::from_v1(&self.render_frame())
    }
}

/// app 侧端点。泛型 [`FrameSource`] 是会话的最小接缝。
pub struct AppEndpoint<S: FrameSource> {
    pub state: AppState,
    /// app 会话（帧生产者 + 输入消费者）。
    pub session: S,
    /// Hello 材料。
    app_name: String,
    title: String,
    width: f32,
    height: f32,
    fonts: Vec<super::message::FontBlob>,
    /// v1.11 多表面声明（壳双表面客户端经 [`Self::with_surfaces`] 注入；
    /// 空 = 单 window 表面，既有行为）。
    surfaces: Vec<super::message::SurfaceDecl>,
    /// 握手结果。
    pub app_id: Option<u64>,
    pub wid: Option<u64>,
    pub surface: Option<u64>,
    pub rect: Option<WRect>,
    /// v1.11 第二及以后表面的分配结果（Welcome 尾段；壳客户端消费）。
    pub extra_surfaces: Vec<super::message::WelcomeSurface>,
    /// 表面帧载荷模式（v1.3：Welcome 协商结果；缺省 Commands）。
    pub frame_mode: FrameMode,
    /// 观测汇（Attach/Detach 维护）。
    pub observe_sink: Option<String>,
    /// 帧序号（单调）。
    next_frame_id: u64,
    /// 上一次发送的槽（双缓冲翻转基线）。
    last_slot: u8,
    /// 宿主归还的空闲槽（观测面）。
    pub free_slots: Vec<u8>,
    /// 槽总数（Welcome 后 = 2）。
    slot_count: u8,
    /// PLAN-034：位图槽（双缓冲翻转基线——BitmapReady/Ack 纪律镜像帧槽）。
    bm_last_slot: u8,
    /// PLAN-034：位图空闲槽（BitmapAck 归还）。
    pub bm_free_slots: Vec<u8>,
}

impl<S: FrameSource> AppEndpoint<S> {
    pub fn new(session: S, app_name: &str, title: &str, width: f32, height: f32) -> Self {
        Self {
            state: AppState::Detached,
            session,
            app_name: app_name.to_string(),
            title: title.to_string(),
            width,
            height,
            fonts: Vec::new(),
            surfaces: Vec::new(),
            app_id: None,
            wid: None,
            surface: None,
            rect: None,
            extra_surfaces: Vec::new(),
            frame_mode: FrameMode::Commands,
            observe_sink: None,
            next_frame_id: 0,
            bm_last_slot: 0,
            bm_free_slots: Vec::new(),
            last_slot: 0,
            free_slots: Vec::new(),
            slot_count: 0,
        }
    }

    /// 注册自带字体（413 §7.2；随 Hello 上传）。
    pub fn with_fonts(mut self, fonts: Vec<super::message::FontBlob>) -> Self {
        self.fonts = fonts;
        self
    }

    /// 注册多表面声明（v1.11 壳双表面客户端；随 Hello 尾段上行）。
    pub fn with_surfaces(mut self, surfaces: Vec<super::message::SurfaceDecl>) -> Self {
        self.surfaces = surfaces;
        self
    }

    /// 发起孵化握手：Detached → Handshaking，产出 Hello。
    /// L2 回归（Standalone → Handshaking）走同一入口：session/revision
    /// 原样保留——revision 不归零即"状态未动"的协议级证据。
    pub fn connect(&mut self) -> Result<ProtocolMsg, ProtocolError> {
        if !matches!(self.state, AppState::Detached | AppState::Standalone) {
            return Err(ProtocolError::WrongState {
                state: self.state.name(),
                msg: "connect()",
            });
        }
        self.state = AppState::Handshaking;
        Ok(ProtocolMsg::Handshake(HandshakeMsg::Hello {
            version: super::PROTOCOL_VERSION,
            app_name: self.app_name.clone(),
            title: self.title.clone(),
            icon: None,
            width: self.width,
            height: self.height,
            fonts: self.fonts.clone(),
            surfaces: self.surfaces.clone(),
        }))
    }

    /// 产一帧：FrameReady（Active 才可；槽 = 空闲优先，否则双缓冲翻转）。
    pub fn produce_frame(&mut self, damage: Option<WRect>) -> Result<ProtocolMsg, ProtocolError> {
        if self.state != AppState::Active {
            return Err(ProtocolError::NotActive);
        }
        let slot = match self.free_slots.pop() {
            Some(s) => s,
            None => 1 - self.last_slot,
        };
        self.last_slot = slot;
        self.next_frame_id += 1;
        let payload = self.session.render_frame();
        Ok(ProtocolMsg::Frame(super::message::FrameMsg::FrameReady {
            wid: self.wid.expect("Active 即有 wid"),
            frame_id: self.next_frame_id,
            slot,
            damage,
            revision: self.session.revision(),
            payload,
        }))
    }

    /// 产一帧走共享内存变体（S9）：载荷编码进 `shm` 的 `slot` 槽，
    /// 管道上只过 FrameReadyShared 元数据。Active 才可。
    pub fn produce_frame_shared(
        &mut self,
        shm: &super::shm::SharedFrameBuffer,
        damage: Option<WRect>,
    ) -> Result<ProtocolMsg, ProtocolError> {
        if self.state != AppState::Active {
            return Err(ProtocolError::NotActive);
        }
        let slot = match self.free_slots.pop() {
            Some(s) => s,
            None => 1 - self.last_slot,
        };
        self.last_slot = slot;
        self.next_frame_id += 1;
        let payload = self.session.render_frame();
        let mut encoded = Vec::new();
        payload.encode(&mut encoded);
        shm.write_slot(slot, &encoded).map_err(ProtocolError::Shm)?;
        let len = encoded.len() as u32;
        Ok(ProtocolMsg::Frame(super::message::FrameMsg::FrameReadyShared {
            wid: self.wid.expect("Active 即有 wid"),
            frame_id: self.next_frame_id,
            slot,
            damage,
            revision: self.session.revision(),
            len,
        }))
    }

    /// 产一帧 v2（PLAN-683 remote 模式）：DisplayList 内嵌帧（信封 =
    /// FrameReadyV2 tag 12）。Active 才可。
    pub fn produce_frame_v2(
        &mut self,
        damage: Option<WRect>,
    ) -> Result<ProtocolMsg, ProtocolError> {
        if self.state != AppState::Active {
            return Err(ProtocolError::NotActive);
        }
        let slot = match self.free_slots.pop() {
            Some(s) => s,
            None => 1 - self.last_slot,
        };
        self.last_slot = slot;
        self.next_frame_id += 1;
        let payload = self.session.render_frame_v2();
        Ok(ProtocolMsg::Frame(super::message::FrameMsg::FrameReadyV2 {
            wid: self.wid.expect("Active 即有 wid"),
            frame_id: self.next_frame_id,
            slot,
            damage,
            revision: self.session.revision(),
            payload,
        }))
    }

    /// 产一帧 v2 走共享内存变体（PLAN-683 remote 模式）：v2 载荷写
    /// `slot` 槽（载荷种类 tag 2），管道上仍过 FrameReadyShared 元数据
    ///（宿主按槽内首字节分派 v1/v2）。Active 才可。
    pub fn produce_frame_shared_v2(
        &mut self,
        shm: &super::shm::SharedFrameBuffer,
        damage: Option<WRect>,
    ) -> Result<ProtocolMsg, ProtocolError> {
        if self.state != AppState::Active {
            return Err(ProtocolError::NotActive);
        }
        let slot = match self.free_slots.pop() {
            Some(s) => s,
            None => 1 - self.last_slot,
        };
        self.last_slot = slot;
        self.next_frame_id += 1;
        let payload = self.session.render_frame_v2();
        let mut encoded = Vec::new();
        payload.encode(&mut encoded);
        shm.write_slot(slot, &encoded).map_err(ProtocolError::Shm)?;
        let len = encoded.len() as u32;
        Ok(ProtocolMsg::Frame(super::message::FrameMsg::FrameReadyShared {
            wid: self.wid.expect("Active 即有 wid"),
            frame_id: self.next_frame_id,
            slot,
            damage,
            revision: self.session.revision(),
            len,
        }))
    }

    /// PLAN-034 T-05（D3）：位图上传产信——RGBA 写位图段 `slot` 槽
    /// （`[u32 len][h×stride 行序列]`），管道上只过 BitmapReady 元数据。
    /// 槽纪律镜像帧槽（空闲优先，否则双缓冲翻转；BitmapAck 归还）。
    /// Active 才可；超槽（位图大于段槽档）Err(Shm)——调用方观测弃置
    /// （v1 显式边界：位图按构造 ≤ 表面尺寸）。
    pub fn produce_bitmap(
        &mut self,
        shm: &super::shm::SharedFrameBuffer,
        up: &BitmapUpload,
    ) -> Result<ProtocolMsg, ProtocolError> {
        if self.state != AppState::Active {
            return Err(ProtocolError::NotActive);
        }
        let slot = match self.bm_free_slots.pop() {
            Some(s) => s,
            None => 1 - self.bm_last_slot,
        };
        self.bm_last_slot = slot;
        shm.write_slot(slot, &up.rgba).map_err(ProtocolError::Shm)?;
        Ok(ProtocolMsg::Frame(super::message::FrameMsg::BitmapReady {
            wid: self.wid.expect("Active 即有 wid"),
            id: up.id.clone(),
            slot,
            w: up.w,
            h: up.h,
            stride: up.stride,
            len: up.rgba.len() as u32,
        }))
    }

    /// 产一帧像素变体（v1.3 independent 臂）：RGBA 写 `shm` 的 `slot` 槽
    /// （统一 `[u32 len][载荷]` 槆框架，载荷解释随 Welcome 模式位），管道
    /// 上只过 FrameReadyPixels 元数据。槽纪律/frame_id 与命令帧同源。
    pub fn produce_frame_pixels(
        &mut self,
        shm: &super::shm::SharedFrameBuffer,
        rgba: &[u8],
        w: u32,
        h: u32,
        stride: u32,
    ) -> Result<ProtocolMsg, ProtocolError> {
        if self.state != AppState::Active {
            return Err(ProtocolError::NotActive);
        }
        let slot = match self.free_slots.pop() {
            Some(s) => s,
            None => 1 - self.last_slot,
        };
        self.last_slot = slot;
        self.next_frame_id += 1;
        shm.write_slot(slot, rgba).map_err(ProtocolError::Shm)?;
        Ok(ProtocolMsg::Frame(super::message::FrameMsg::FrameReadyPixels {
            wid: self.wid.expect("Active 即有 wid"),
            frame_id: self.next_frame_id,
            slot,
            damage: None, // v1.3：damage 作提示（全帧有效）
            revision: self.session.revision(),
            w,
            h,
            stride,
            format: super::message::PixelFormat::Rgba8,
        }))
    }

    /// app 主动请求退出：Active → Closing，产出 ExitRequest。
    pub fn send_exit(&mut self) -> Result<ProtocolMsg, ProtocolError> {
        if self.state != AppState::Active {
            return Err(ProtocolError::WrongState {
                state: self.state.name(),
                msg: "send_exit()",
            });
        }
        self.state = AppState::Closing;
        Ok(ProtocolMsg::Control(ControlMsg::ExitRequest {
            wid: self.wid.expect("Active 即有 wid"),
        }))
    }

    /// app 侧状态机：处理一条来自宿主的消息，返回要回发的消息序列。
    pub fn on_message(&mut self, msg: ProtocolMsg) -> Result<Vec<ProtocolMsg>, ProtocolError> {
        use AppState::*;
        match (&self.state, msg) {
            // --- 握手 ---
            (Handshaking, ProtocolMsg::Handshake(HandshakeMsg::Welcome { app_id, wid, surface, rect, frame_mode, extra_surfaces })) => {
                self.app_id = Some(app_id);
                self.wid = Some(wid);
                self.surface = Some(surface);
                self.rect = Some(rect);
                self.frame_mode = frame_mode;
                self.extra_surfaces = extra_surfaces;
                self.state = Active;
                self.slot_count = 2;
                self.free_slots = vec![1]; // 槽 0 视为在写首帧
                self.last_slot = 0;
                self.bm_last_slot = 0;
                self.bm_free_slots.clear();
                Ok(vec![ProtocolMsg::Handshake(HandshakeMsg::Ready)])
            }
            // --- Active：四通道日常 ---
            (Active, ProtocolMsg::Input(input)) => {
                self.session.on_input(&input);
                Ok(vec![])
            }
            (Active, ProtocolMsg::Control(control)) => {
                match control {
                    ControlMsg::Close { .. } => {
                        self.session.on_control(&control);
                        self.state = Closing;
                        Ok(vec![ProtocolMsg::Control(ControlMsg::ExitRequest {
                            wid: self.wid.expect("Active 即有 wid"),
                        })])
                    }
                    ControlMsg::L2Detach { .. } => {
                        // L2"独立出去"：会话/状态不动，表面交还宿主；
                        // app 进程此后自管渲染（Stage 2 = 自开 OS 窗）。
                        self.session.on_control(&control);
                        self.state = AppState::Standalone;
                        Ok(vec![ProtocolMsg::Control(ControlMsg::L2Detached {
                            wid: self.wid.expect("Active 即有 wid"),
                        })])
                    }
                    _ => {
                        self.session.on_control(&control);
                        Ok(vec![])
                    }
                }
            }
            (Active, ProtocolMsg::Frame(super::message::FrameMsg::FrameAck { slot, .. })) => {
                self.free_slots.push(slot);
                Ok(vec![])
            }
            // PLAN-034（D3）：位图槽归还——同 FrameAck 纪律。
            (Active, ProtocolMsg::Frame(super::message::FrameMsg::BitmapAck { slot, .. })) => {
                self.bm_free_slots.push(slot);
                Ok(vec![])
            }
            // 握手尾随的缓冲分配 / 宿主侧 resize 重协商：登记即接受。
            (Active, ProtocolMsg::Frame(super::message::FrameMsg::BufferAlloc { slots, .. })) => {
                self.slot_count = slots;
                Ok(vec![])
            }
            (Active, ProtocolMsg::Frame(super::message::FrameMsg::Resize { .. })) => Ok(vec![]),
            (Active, ProtocolMsg::Frame(super::message::FrameMsg::BufferRelease { .. })) => {
                // 宿主回收缓冲 = 摘除表面（常见于 detach 前奏）。
                self.state = Detached;
                self.slot_count = 0;
                self.free_slots.clear();
                self.bm_free_slots.clear();
                Ok(vec![])
            }
            (Active, ProtocolMsg::Observe(ob)) => {
                match ob {
                    ObserveMsg::Attach { sink, .. } => self.observe_sink = Some(sink),
                    ObserveMsg::Detach { .. } => self.observe_sink = None,
                    _ => {}
                }
                Ok(vec![])
            }
            // --- Closing：等回收 ---
            (Closing, ProtocolMsg::Frame(super::message::FrameMsg::BufferRelease { .. })) => {
                self.state = Detached;
                self.slot_count = 0;
                self.free_slots.clear();
                self.bm_free_slots.clear();
                Ok(vec![])
            }
            // --- 其余一律非法 ---
            (state, msg) => Err(ProtocolError::WrongState {
                state: state.name(),
                msg: msg_name(&msg),
            }),
        }
    }

    /// 槽总数（握手后 = 2）。
    pub fn slot_count(&self) -> u8 {
        self.slot_count
    }
}

// ---------------------------------------------------------------------------
// host 侧
// ---------------------------------------------------------------------------

/// Host 端点状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostState {
    /// 等 Hello。
    Listening,
    /// 一路孵化已就位（Stage 1 loopback = 单客户端；多 App = Stage 3）。
    Active,
}

/// 宿主侧需要会话适配层执行的动作（`host::ProtocolHost` 的落点）。
#[derive(Debug, Clone, PartialEq)]
pub enum HostAction {
    /// 收到合法 Hello：解析 app_name → 编译装载 → allocate + wm_add_win，
    /// 然后回调 [`HostEndpoint::activate`] 取 Welcome 回发。
    /// `surfaces`（v1.11）：多表面声明（空 = 单 window 既有行为）。
    ResolveAndAttach {
        version: u16,
        app_name: String,
        title: String,
        width: f32,
        height: f32,
        fonts: Vec<super::message::FontBlob>,
        surfaces: Vec<super::message::SurfaceDecl>,
    },
    /// 帧合成：写入 surface 的 slot（双缓冲翻面）；适配层随合成回 FrameAck
    /// （frame_id 原样回带）。
    ComposeFrame { surface: u64, wid: u64, frame_id: u64, slot: u8, revision: u64, payload: DrawList },
    /// 共享内存变体（S9）：适配层从 shm 槽读载荷解码后合成。
    ComposeFrameShared { surface: u64, wid: u64, frame_id: u64, slot: u8, revision: u64, len: u32 },
    /// v2 内嵌帧（PLAN-683 remote 模式）：DisplayList 载荷直达（无 shm），
    /// 适配层入 v2 槽合成 + 回 FrameAck。
    ComposeFrameV2 {
        surface: u64,
        wid: u64,
        frame_id: u64,
        slot: u8,
        revision: u64,
        payload: super::message::DisplayList,
    },
    /// 像素帧变体（v1.3）：适配层从 shm 槽读 RGBA 上传合成
    /// （independent 臂；载荷解释随 Welcome 模式位）。
    ComposeFramePixels {
        surface: u64,
        wid: u64,
        frame_id: u64,
        slot: u8,
        revision: u64,
        w: u32,
        h: u32,
        stride: u32,
    },
    /// app 确认退出/请求退出：回收虚拟窗（462 Close 语义）。
    ReclaimWindow { wid: u64 },
    /// PLAN-034 T-05（D3）：app 位图就绪——适配层从位图段读 RGBA 入宿主
    /// 图像缓存（`bitmap://{id}` 键）后回 `BitmapAck` 归还槽。
    BitmapReady {
        surface: u64,
        wid: u64,
        id: String,
        w: u32,
        h: u32,
        stride: u32,
        slot: u8,
        len: u32,
    },
    /// app→host 命令上行（v1.11 落地）：`record` = `DesktopCommand` 单记录
    /// 编码串（`verb␟arg`）——适配层 parse_records 单点解析执行；归因按
    /// wid → registry_id。此前在端点丢弃臂中无声消失（desktop.* wire 化
    /// 缺口，PLAN-030 D4 清偿）。
    DesktopBus { wid: u64, record: String },
    /// 观测上行转发（MCP 代理的最小落点）。
    ObserveUp { msg: ObserveMsg },
}

/// host 侧端点（单客户端；多 App 并发归 Stage 3）。
pub struct HostEndpoint {
    pub state: HostState,
    /// 当前客户端（Active 后有效）。
    pub app_id: Option<u64>,
    pub wid: Option<u64>,
    pub surface: Option<u64>,
    /// v1.11 第二及以后表面（多表面客户端；帧路由按 wid 查此表）。
    pub extra_surfaces: Vec<(u64, u64)>,
    /// 表面帧载荷模式（v1.3：activate 时随裁决结果定档）。
    pub frame_mode: FrameMode,
}

impl HostEndpoint {
    pub fn listen() -> Self {
        Self {
            state: HostState::Listening,
            app_id: None,
            wid: None,
            surface: None,
            extra_surfaces: Vec::new(),
            frame_mode: FrameMode::Commands,
        }
    }

    /// Active 帧路由：消息 wid → surface 句柄（多表面客户端第二及以后
    /// 表面查 `extra_surfaces`；主表面走既有单值字段）。
    fn surface_for(&self, wid: u64) -> Option<u64> {
        if self.wid == Some(wid) {
            self.surface
        } else {
            self.extra_surfaces.iter().find(|(w, _)| *w == wid).map(|(_, s)| *s)
        }
    }

    /// 宿主侧状态机：处理一条来自 app 的消息，产出动作序列。
    pub fn on_message(&mut self, msg: ProtocolMsg) -> Result<Vec<HostAction>, ProtocolError> {
        match (&self.state, msg) {
            // --- 孵化：Hello → ResolveAndAttach ---
            (
                HostState::Listening,
                ProtocolMsg::Handshake(HandshakeMsg::Hello {
                    version,
                    app_name,
                    title,
                    icon: _,
                    width,
                    height,
                    fonts,
                    surfaces,
                }),
            ) => {
                if version != super::PROTOCOL_VERSION {
                    return Err(ProtocolError::VersionMismatch(version));
                }
                if app_name.is_empty() {
                    return Err(ProtocolError::WrongState { state: "Listening", msg: "empty app_name" });
                }
                Ok(vec![HostAction::ResolveAndAttach { version, app_name, title, width, height, fonts, surfaces }])
            }
            // --- Active：帧/输入确认/控制/观测 ---
            (
                HostState::Active,
                ProtocolMsg::Frame(super::message::FrameMsg::FrameReady {
                    wid,
                    frame_id,
                    slot,
                    damage: _,
                    revision,
                    payload,
                }),
            ) => {
                let surface = self.surface_for(wid).expect("Active 即有 surface");
                Ok(vec![HostAction::ComposeFrame {
                    surface,
                    wid,
                    frame_id,
                    slot,
                    revision,
                    payload,
                }])
            }
            (
                HostState::Active,
                ProtocolMsg::Frame(super::message::FrameMsg::FrameReadyShared {
                    wid,
                    frame_id,
                    slot,
                    damage: _,
                    revision,
                    len,
                }),
            ) => {
                let surface = self.surface_for(wid).expect("Active 即有 surface");
                Ok(vec![HostAction::ComposeFrameShared {
                    surface,
                    wid,
                    frame_id,
                    slot,
                    revision,
                    len,
                }])
            }
            // v1.3 independent 臂：像素帧元数据 → 适配层 shm 读 RGBA 上传。
            (
                HostState::Active,
                ProtocolMsg::Frame(super::message::FrameMsg::FrameReadyPixels {
                    wid,
                    frame_id,
                    slot,
                    damage: _,
                    revision,
                    w,
                    h,
                    stride,
                    format: _,
                }),
            ) => {
                let surface = self.surface_for(wid).expect("Active 即有 surface");
                Ok(vec![HostAction::ComposeFramePixels {
                    surface,
                    wid,
                    frame_id,
                    slot,
                    revision,
                    w,
                    h,
                    stride,
                }])
            }
            // PLAN-683（remote 模式）：v2 内嵌帧 → 适配层直接合成（无 shm）。
            (
                HostState::Active,
                ProtocolMsg::Frame(super::message::FrameMsg::FrameReadyV2 {
                    wid,
                    frame_id,
                    slot,
                    damage: _,
                    revision,
                    payload,
                }),
            ) => {
                let surface = self.surface_for(wid).expect("Active 即有 surface");
                Ok(vec![HostAction::ComposeFrameV2 {
                    surface,
                    wid,
                    frame_id,
                    slot,
                    revision,
                    payload,
                }])
            }
            // PLAN-034 T-05（D3）：位图就绪——surface 解析同 ComposeFrame
            // （wid 路由），适配层读槽入缓存 + 回 BitmapAck。
            (
                HostState::Active,
                ProtocolMsg::Frame(super::message::FrameMsg::BitmapReady {
                    wid,
                    id,
                    slot,
                    w,
                    h,
                    stride,
                    len,
                }),
            ) => {
                let surface = self.surface_for(wid).expect("Active 即有 surface");
                Ok(vec![HostAction::BitmapReady { surface, wid, id, w, h, stride, slot, len }])
            }
            // 握手确认（app 的 Ready）：Active 后的例行收尾，无动作。
            (HostState::Active, ProtocolMsg::Handshake(HandshakeMsg::Ready)) => Ok(vec![]),
            // Plan 508：HitTable 为宿主→远程镜像端变体，宿主端点不产不出、
            // 收到按未知方向拒绝（pipe/loopback 会话不经过此路径）。
            (_, ProtocolMsg::Frame(super::message::FrameMsg::HitTable { .. })) => {
                Err(ProtocolError::WrongState {
                    state: "host-endpoint",
                    msg: "Frame::HitTable",
                })
            }
            (HostState::Active, ProtocolMsg::Control(ControlMsg::ExitRequest { wid })) => {
                self.state = HostState::Listening;
                self.app_id = None;
                self.wid = None;
                self.surface = None;
                self.extra_surfaces = Vec::new();
                Ok(vec![HostAction::ReclaimWindow { wid }])
            }
            // L2 确认：app 已切自管表面 → 回收虚拟窗（462 Close 语义），
            // 回 Listening 等 L2 重挂（Hello 复用孵化路径）。
            (HostState::Active, ProtocolMsg::Control(ControlMsg::L2Detached { wid })) => {
                self.state = HostState::Listening;
                self.app_id = None;
                self.wid = None;
                self.surface = None;
                self.extra_surfaces = Vec::new();
                Ok(vec![HostAction::ReclaimWindow { wid }])
            }
            // L2 重挂意向（信息性；真正的孵化凭随后的 Hello）。
            (_, ProtocolMsg::Control(ControlMsg::L2AttachRequest { .. })) => Ok(vec![]),
            // v1.11（PLAN-030 D4）：命令上行拆出执行臂——此前与
            // TitleChanged/Notify 同在丢弃臂（假透传，desktop.* wire 化
            // 缺口）。Title/Notify 维持端点丢弃（落点在适配层，另行清偿）。
            (HostState::Active, ProtocolMsg::Control(ControlMsg::DesktopBus { wid, record })) => {
                Ok(vec![HostAction::DesktopBus { wid, record }])
            }
            (HostState::Active, ProtocolMsg::Control(control @ (ControlMsg::TitleChanged { .. } | ControlMsg::Notify { .. }))) => {
                // 控制上行在此端点只做透传记录；落点在适配层（title→chrome、
                // notify→通知中心）。
                let _ = control;
                Ok(vec![])
            }
            (HostState::Active, ProtocolMsg::Observe(ob)) => {
                Ok(vec![HostAction::ObserveUp { msg: ob }])
            }
            (state, msg) => Err(ProtocolError::WrongState {
                state: host_name(state),
                msg: msg_name(&msg),
            }),
        }
    }

    /// 适配层完成 allocate + wm_add_win + surface 分配后回调：登记客户端
    /// 并产出 Welcome（回发 app）。`frame_mode`：v1.3 表面帧载荷模式
    /// （宿主按 per-App 裁决结果下发；缺省 Commands = 既有行为）。
    pub fn activate(
        &mut self,
        app_id: u64,
        wid: u64,
        surface: u64,
        rect: WRect,
        frame_mode: FrameMode,
    ) -> Result<ProtocolMsg, ProtocolError> {
        if self.state != HostState::Listening {
            return Err(ProtocolError::WrongState {
                state: host_name(&self.state),
                msg: "activate()",
            });
        }
        self.state = HostState::Active;
        self.app_id = Some(app_id);
        self.wid = Some(wid);
        self.surface = Some(surface);
        self.frame_mode = frame_mode;
        Ok(ProtocolMsg::Handshake(HandshakeMsg::Welcome {
            app_id,
            wid,
            surface,
            rect,
            frame_mode,
            extra_surfaces: Vec::new(),
        }))
    }

    /// 多表面版 activate（v1.11 壳双表面客户端）：首表面走既有领头
    /// 字段，其余表面随 Welcome 尾段；端点登记 wid→surface 路由表。
    pub fn activate_multi(
        &mut self,
        app_id: u64,
        wid: u64,
        surface: u64,
        rect: WRect,
        frame_mode: FrameMode,
        extras: Vec<super::message::WelcomeSurface>,
    ) -> Result<ProtocolMsg, ProtocolError> {
        if self.state != HostState::Listening {
            return Err(ProtocolError::WrongState {
                state: host_name(&self.state),
                msg: "activate_multi()",
            });
        }
        self.state = HostState::Active;
        self.app_id = Some(app_id);
        self.wid = Some(wid);
        self.surface = Some(surface);
        self.extra_surfaces = extras.iter().map(|e| (e.wid, e.surface)).collect();
        self.frame_mode = frame_mode;
        Ok(ProtocolMsg::Handshake(HandshakeMsg::Welcome {
            app_id,
            wid,
            surface,
            rect,
            frame_mode,
            extra_surfaces: extras,
        }))
    }

    /// L2"独立出去"：产出 L2Detach（发往 app；回收等 L2Detached 回来）。
    pub fn l2_detach(&self) -> Result<ProtocolMsg, ProtocolError> {
        if self.state != HostState::Active {
            return Err(ProtocolError::WrongState {
                state: host_name(&self.state),
                msg: "l2_detach()",
            });
        }
        Ok(ProtocolMsg::Control(ControlMsg::L2Detach {
            wid: self.wid.expect("Active 即有 wid"),
        }))
    }

    /// 宿主主动关闭：产出 Close（发往 app；回收等 ExitRequest 回来）。
    pub fn close(&self) -> Result<ProtocolMsg, ProtocolError> {
        if self.state != HostState::Active {
            return Err(ProtocolError::WrongState {
                state: host_name(&self.state),
                msg: "close()",
            });
        }
        Ok(ProtocolMsg::Control(ControlMsg::Close {
            wid: self.wid.expect("Active 即有 wid"),
        }))
    }
}

fn host_name(s: &HostState) -> &'static str {
    match s {
        HostState::Listening => "Host::Listening",
        HostState::Active => "Host::Active",
    }
}

/// 消息的短名（错误上下文用）。
fn msg_name(msg: &ProtocolMsg) -> &'static str {
    match msg {
        ProtocolMsg::Handshake(m) => match m {
            HandshakeMsg::Hello { .. } => "Handshake::Hello",
            HandshakeMsg::Welcome { .. } => "Handshake::Welcome",
            HandshakeMsg::Ready => "Handshake::Ready",
        },
        ProtocolMsg::Frame(m) => match m {
            super::message::FrameMsg::BufferAlloc { .. } => "Frame::BufferAlloc",
            super::message::FrameMsg::BufferRelease { .. } => "Frame::BufferRelease",
            super::message::FrameMsg::Resize { .. } => "Frame::Resize",
            super::message::FrameMsg::FrameReady { .. } => "Frame::FrameReady",
            super::message::FrameMsg::FrameAck { .. } => "Frame::FrameAck",
            super::message::FrameMsg::CacheControl { .. } => "Frame::CacheControl",
            super::message::FrameMsg::FrameReadyShared { .. } => "Frame::FrameReadyShared",
            super::message::FrameMsg::FrameReadyPixels { .. } => "Frame::FrameReadyPixels",
            super::message::FrameMsg::HitTable { .. } => "Frame::HitTable",
            super::message::FrameMsg::BitmapReady { .. } => "Frame::BitmapReady",
            super::message::FrameMsg::BitmapAck { .. } => "Frame::BitmapAck",
            super::message::FrameMsg::FrameReadyV2 { .. } => "Frame::FrameReadyV2",
        },
        ProtocolMsg::Input(_) => "Input",
        ProtocolMsg::Control(m) => match m {
            ControlMsg::Close { .. } => "Control::Close",
            ControlMsg::Focus { .. } => "Control::Focus",
            ControlMsg::Resize { .. } => "Control::Resize",
            ControlMsg::TitleChanged { .. } => "Control::TitleChanged",
            ControlMsg::Notify { .. } => "Control::Notify",
            ControlMsg::ExitRequest { .. } => "Control::ExitRequest",
            ControlMsg::DesktopBus { .. } => "Control::DesktopBus",
            ControlMsg::L2Detach { .. } => "Control::L2Detach",
            ControlMsg::L2Detached { .. } => "Control::L2Detached",
            ControlMsg::L2AttachRequest { .. } => "Control::L2AttachRequest",
            ControlMsg::StateSnapshot { .. } => "Control::StateSnapshot",
            ControlMsg::ShellProjectionPush { .. } => "Control::ShellProjectionPush",
            ControlMsg::ShellClockTick { .. } => "Control::ShellClockTick",
            ControlMsg::ShellCursorMove { .. } => "Control::ShellCursorMove",
        },
        ProtocolMsg::Observe(m) => match m {
            ObserveMsg::Attach { .. } => "Observe::Attach",
            ObserveMsg::Detach { .. } => "Observe::Detach",
            ObserveMsg::Log { .. } => "Observe::Log",
            ObserveMsg::Metric { .. } => "Observe::Metric",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::desktop_protocol::message::{DrawOp, FrameMsg, MouseButton, Rgba8};

    /// 计数器会话桩：点击计数 + 产一帧文本。
    struct StubSource {
        count: u64,
        rev: u64,
        last_input: Option<InputMsg>,
    }

    impl StubSource {
        fn new() -> Self {
            Self { count: 0, rev: 1, last_input: None }
        }
    }

    impl FrameSource for StubSource {
        fn revision(&self) -> u64 {
            self.rev
        }

        fn render_frame(&mut self) -> DrawList {
            DrawList {
                clear: Some(Rgba8::new(20, 20, 24, 255)),
                ops: vec![DrawOp::Text {
                    x: 8.0,
                    y: 8.0,
                    size: 14.0,
                    line_height: 20.0,
                    color: Rgba8::new(255, 255, 255, 255),
                    text: format!("count: {}", self.count),
                }],
            }
        }

        fn on_input(&mut self, input: &InputMsg) {
            if matches!(input, InputMsg::PointerPressed { button: MouseButton::Left, .. }) {
                self.count += 1;
                self.rev += 1;
            }
            self.last_input = Some(input.clone());
        }

        fn on_control(&mut self, _control: &ControlMsg) {}
    }

    /// 握手到 Active 的公共前奏。
    fn activate_pair(
        app: &mut AppEndpoint<StubSource>,
        host: &mut HostEndpoint,
    ) -> Vec<ProtocolMsg> {
        let hello = app.connect().unwrap();
        let actions = host.on_message(hello).unwrap();
        assert!(matches!(actions[0], HostAction::ResolveAndAttach { .. }));
        let welcome = host
            .activate(1, 3, 42, WRect::new(16.0, 16.0, 480.0, 320.0), FrameMode::Commands)
            .unwrap();
        let replies = app.on_message(welcome).unwrap();
        assert_eq!(replies, vec![ProtocolMsg::Handshake(HandshakeMsg::Ready)]);
        replies
    }

    #[test]
    fn happy_path_handshake_to_active() {
        let mut app = AppEndpoint::new(StubSource::new(), "counter", "计数器", 480.0, 320.0);
        let mut host = HostEndpoint::listen();

        assert_eq!(app.state, AppState::Detached);
        let hello = app.connect().unwrap();
        assert_eq!(app.state, AppState::Handshaking);
        assert!(matches!(
            hello,
            ProtocolMsg::Handshake(HandshakeMsg::Hello { version: 1, .. })
        ));

        let actions = host.on_message(hello).unwrap();
        let HostAction::ResolveAndAttach { app_name, title, width, height, .. } =
            &actions[0]
        else {
            panic!("期待 ResolveAndAttach");
        };
        assert_eq!(app_name, "counter");
        assert_eq!(title, "计数器");
        assert_eq!((*width, *height), (480.0, 320.0));

        let welcome = host
            .activate(1, 3, 42, WRect::new(16.0, 16.0, 480.0, 320.0), FrameMode::Commands)
            .unwrap();
        assert_eq!(host.state, HostState::Active);
        assert_eq!((host.app_id, host.wid, host.surface), (Some(1), Some(3), Some(42)));

        let replies = app.on_message(welcome).unwrap();
        assert_eq!(replies.len(), 1);
        assert_eq!(replies[0], ProtocolMsg::Handshake(HandshakeMsg::Ready));
        assert_eq!(app.state, AppState::Active);
        assert_eq!((app.app_id, app.wid, app.surface), (Some(1), Some(3), Some(42)));
    }

    #[test]
    fn frame_and_ack_double_buffer_cycle() {
        let mut app = AppEndpoint::new(StubSource::new(), "counter", "c", 480.0, 320.0);
        let mut host = HostEndpoint::listen();
        let ready = activate_pair(&mut app, &mut host);
        let _ = ready;

        // 帧 0：无空闲槽 → 翻转出槽 1（last_slot=0）。
        let f0 = app.produce_frame(None).unwrap();
        let ProtocolMsg::Frame(FrameMsg::FrameReady { wid, frame_id, slot, revision, payload, .. }) =
            &f0
        else {
            panic!("期待 FrameReady");
        };
        assert_eq!((*wid, *frame_id, *slot, *revision), (3, 1, 1, 1));
        assert!(matches!(payload.ops[0], DrawOp::Text { .. }));

        // 宿主合成 + 归还槽 1。
        let actions = host.on_message(f0).unwrap();
        assert!(matches!(actions[0], HostAction::ComposeFrame { surface: 42, slot: 1, .. }));
        let ack = ProtocolMsg::Frame(FrameMsg::FrameAck { wid: 3, frame_id: 1, slot: 1 });
        app.on_message(ack).unwrap();
        assert_eq!(app.free_slots, vec![1], "槽归还空闲池");

        // 帧 1：优先用空闲槽 1。
        let f1 = app.produce_frame(None).unwrap();
        let ProtocolMsg::Frame(FrameMsg::FrameReady { frame_id, slot, .. }) = &f1 else {
            panic!()
        };
        assert_eq!((*frame_id, *slot), (2, 1));
    }

    #[test]
    fn input_dispatch_increments_session() {
        let mut app = AppEndpoint::new(StubSource::new(), "counter", "c", 480.0, 320.0);
        let mut host = HostEndpoint::listen();
        let _ = activate_pair(&mut app, &mut host);

        let click = ProtocolMsg::Input(InputMsg::PointerPressed {
            wid: 3,
            button: MouseButton::Left,
            x: 60.0,
            y: 20.0,
            modifiers: 0,
        });
        app.on_message(click).unwrap();
        assert_eq!(app.session.count, 1, "输入经状态机到达会话");
        assert_eq!(app.session.revision(), 2, "状态变化推版本");
    }

    #[test]
    fn host_close_then_exit_handshake_then_reclaim() {
        let mut app = AppEndpoint::new(StubSource::new(), "counter", "c", 480.0, 320.0);
        let mut host = HostEndpoint::listen();
        let _ = activate_pair(&mut app, &mut host);

        // 宿主主动 Close → app 转 Closing + 回 ExitRequest。
        let close = host.close().unwrap();
        assert!(matches!(close, ProtocolMsg::Control(ControlMsg::Close { wid: 3 })));
        let replies = app.on_message(close).unwrap();
        assert_eq!(app.state, AppState::Closing);
        assert_eq!(replies, vec![ProtocolMsg::Control(ControlMsg::ExitRequest { wid: 3 })]);

        // 宿主收 ExitRequest → ReclaimWindow + 回 Listening。
        let actions = host.on_message(replies[0].clone()).unwrap();
        assert_eq!(actions, vec![HostAction::ReclaimWindow { wid: 3 }]);
        assert_eq!(host.state, HostState::Listening);
        assert_eq!(host.wid, None);

        // 回收确认（BufferRelease）→ app 回 Detached，可再 connect。
        app.on_message(ProtocolMsg::Frame(FrameMsg::BufferRelease { surface: 42 })).unwrap();
        assert_eq!(app.state, AppState::Detached);
        assert!(app.connect().is_ok(), "回收后可重新孵化");
    }

    #[test]
    fn observe_attach_and_uplink() {
        let mut app = AppEndpoint::new(StubSource::new(), "counter", "c", 480.0, 320.0);
        let mut host = HostEndpoint::listen();
        let _ = activate_pair(&mut app, &mut host);

        app.on_message(ProtocolMsg::Observe(ObserveMsg::Attach {
            wid: 3,
            sink: "mcp://desktop/app-3".into(),
        }))
        .unwrap();
        assert_eq!(app.observe_sink.as_deref(), Some("mcp://desktop/app-3"));

        let actions = host
            .on_message(ProtocolMsg::Observe(ObserveMsg::Log {
                wid: 3,
                level: super::super::message::LogLevel::Info,
                message: "ready".into(),
            }))
            .unwrap();
        assert_eq!(
            actions,
            vec![HostAction::ObserveUp {
                msg: ObserveMsg::Log { wid: 3, level: super::super::message::LogLevel::Info, message: "ready".into() }
            }]
        );
    }

    #[test]
    fn l2_detach_attach_round_trip_state_continuous() {
        let mut app = AppEndpoint::new(StubSource::new(), "counter", "c", 480.0, 320.0);
        let mut host = HostEndpoint::listen();
        let _ = activate_pair(&mut app, &mut host);

        // 活跃期点击两次 → revision 前进。
        for _ in 0..2 {
            app.on_message(ProtocolMsg::Input(InputMsg::PointerPressed {
                wid: 3,
                button: MouseButton::Left,
                x: 1.0,
                y: 1.0,
                modifiers: 0,
            }))
            .unwrap();
        }
        let rev_before = app.session.revision();
        assert_eq!(rev_before, 3);

        // L2 独立出去：宿主 L2Detach → app Standalone + L2Detached 确认
        // → 宿主 ReclaimWindow + 回 Listening。
        let detach = host.l2_detach().unwrap();
        let replies = app.on_message(detach).unwrap();
        assert_eq!(app.state, AppState::Standalone);
        assert_eq!(
            replies,
            vec![ProtocolMsg::Control(ControlMsg::L2Detached { wid: 3 })]
        );
        let actions = host.on_message(replies[0].clone()).unwrap();
        assert_eq!(actions, vec![HostAction::ReclaimWindow { wid: 3 }]);
        assert_eq!(host.state, HostState::Listening);

        // Standalone 期间产帧拒绝（表面已交还）。
        assert_eq!(app.produce_frame(None), Err(ProtocolError::NotActive));

        // L2 重挂：connect() 从 Standalone 走同一握手，session 原样。
        let hello = app.connect().unwrap();
        assert_eq!(app.state, AppState::Handshaking);
        let actions = host.on_message(hello).unwrap();
        assert!(matches!(actions[0], HostAction::ResolveAndAttach { .. }));
        // 新 wid/surface（宿主重新分配）。
        let welcome = host
            .activate(1, 9, 77, WRect::new(16.0, 16.0, 480.0, 320.0), FrameMode::Commands)
            .unwrap();
        app.on_message(welcome).unwrap();
        assert_eq!(app.state, AppState::Active);
        assert_eq!(app.wid, Some(9));
        assert_eq!(app.app_id, Some(1));

        // revision 连续 = 状态未动的协议级证据。
        assert_eq!(app.session.revision(), rev_before, "L2 往返 revision 不归零");
        // 且会话仍可产帧/响应输入。
        let f = app.produce_frame(None).unwrap();
        let ProtocolMsg::Frame(FrameMsg::FrameReady { revision, .. }) = &f else {
            panic!("期待 FrameReady");
        };
        assert_eq!(*revision, rev_before);
    }

    #[test]
    fn illegal_transitions_rejected() {
        let mut app = AppEndpoint::new(StubSource::new(), "counter", "c", 480.0, 320.0);
        let mut host = HostEndpoint::listen();

        // Detached 状态产帧 / 收 Welcome / 二次 connect 均拒。
        assert_eq!(app.produce_frame(None), Err(ProtocolError::NotActive));
        assert_eq!(
            app.on_message(ProtocolMsg::Handshake(HandshakeMsg::Welcome {
                app_id: 1,
                wid: 1,
                surface: 1,
                rect: WRect::default(),
                frame_mode: FrameMode::Commands,
                extra_surfaces: Vec::new(),
            })),
            Err(ProtocolError::WrongState { state: "Detached", msg: "Handshake::Welcome" })
        );
        assert!(app.connect().is_ok());
        assert_eq!(
            app.connect(),
            Err(ProtocolError::WrongState { state: "Handshaking", msg: "connect()" })
        );

        // Active 前宿主收不到帧；Listening 状态不能 close/activate 两次。
        assert_eq!(
            host.on_message(ProtocolMsg::Frame(FrameMsg::FrameReady {
                wid: 1,
                frame_id: 1,
                slot: 0,
                damage: None,
                revision: 1,
                payload: DrawList::default()
            })),
            Err(ProtocolError::WrongState { state: "Host::Listening", msg: "Frame::FrameReady" })
        );
        assert!(matches!(host.on_message(ProtocolMsg::Handshake(HandshakeMsg::Ready)),
            Err(ProtocolError::WrongState { state: "Host::Listening", msg: "Handshake::Ready" })));
        // 合法前奏在全新端点上重演（上面的 app 已停在 Handshaking——非法
        // 迁移不改状态，但它的 connect 已被消费）。
        let mut app2 = AppEndpoint::new(StubSource::new(), "counter", "c", 480.0, 320.0);
        let mut host2 = HostEndpoint::listen();
        let _ = activate_pair(&mut app2, &mut host2);
        assert!(matches!(host2.activate(2, 4, 43, WRect::default(), FrameMode::Commands),
            Err(ProtocolError::WrongState { state: "Host::Active", msg: "activate()" })));
        // 版本不符拒收。
        let mut host2 = HostEndpoint::listen();
        let mismatch = host2.on_message(ProtocolMsg::Handshake(HandshakeMsg::Hello {
            version: 99,
            app_name: "x".into(),
            title: "x".into(),
            icon: None,
            width: 10.0,
            height: 10.0,
            fonts: vec![],
            surfaces: Vec::new(),
        }));
        assert_eq!(mismatch, Err(ProtocolError::VersionMismatch(99)));
    }

    /// PLAN-030 T-02（D4）：DesktopBus 上行拆出执行臂——此前与
    /// Title/Notify 同在丢弃臂（`let _ = control`）无声消失。
    #[test]
    fn desktop_bus_uplink_produces_action() {
        use super::super::message::{ControlMsg, ProtocolMsg};
        let mut host = HostEndpoint::listen();
        let mut app = AppEndpoint::new(StubSource::new(), "counter", "c", 480.0, 320.0);
        activate_pair(&mut app, &mut host);

        // Title/Notify 维持丢弃（落点另行清偿）；DesktopBus 产动作。
        let up = host
            .on_message(ProtocolMsg::Control(ControlMsg::DesktopBus {
                wid: 3,
                record: "launch\u{1f}counter".into(),
            }))
            .unwrap();
        assert_eq!(
            up,
            vec![HostAction::DesktopBus { wid: 3, record: "launch\u{1f}counter".into() }]
        );
        let drop_title = host
            .on_message(ProtocolMsg::Control(ControlMsg::TitleChanged {
                wid: 3,
                title: "t".into(),
            }))
            .unwrap();
        assert!(drop_title.is_empty());
    }

    /// PLAN-030 T-02（D1）：多表面握手——Hello 尾段声明 → ResolveAndAttach
    /// 携带 → activate_multi 登记 wid→surface 路由 + Welcome 尾段；第二
    /// 表面帧按消息 wid 路由到对应 surface。
    #[test]
    fn multi_surface_handshake_and_frame_routing() {
        use super::super::message::{
            surface_role, ControlMsg, FrameMsg, ProtocolMsg, SurfaceDecl, WelcomeSurface,
        };
        let mut host = HostEndpoint::listen();

        // 壳 Hello：双表面声明（background 全屏 + chrome 任务栏带）。
        let hello = ProtocolMsg::Handshake(HandshakeMsg::Hello {
            version: super::super::PROTOCOL_VERSION,
            app_name: "shell".into(),
            title: "shell".into(),
            icon: None,
            width: 1280.0,
            height: 800.0,
            fonts: vec![],
            surfaces: vec![
                SurfaceDecl { role: surface_role::BACKGROUND, width: 1280.0, height: 800.0 },
                SurfaceDecl { role: surface_role::CHROME, width: 1280.0, height: 48.0 },
            ],
        });
        let actions = host.on_message(hello).unwrap();
        match &actions[0] {
            HostAction::ResolveAndAttach { app_name, surfaces, .. } => {
                assert_eq!(app_name, "shell");
                assert_eq!(surfaces.len(), 2);
                assert_eq!(surfaces[1].role, surface_role::CHROME);
            }
            other => panic!("期待 ResolveAndAttach，得到 {other:?}"),
        }

        // 适配层分配两窗两表面后 activate_multi：首表面领头 + 尾段第二表面。
        let welcome = host
            .activate_multi(
                9,
                100,
                900,
                WRect::new(0.0, 0.0, 1280.0, 800.0),
                FrameMode::Commands,
                vec![WelcomeSurface {
                    role: surface_role::CHROME,
                    wid: 101,
                    surface: 901,
                    rect: WRect::new(0.0, 752.0, 1280.0, 48.0),
                }],
            )
            .unwrap();
        let ProtocolMsg::Handshake(HandshakeMsg::Welcome { wid, extra_surfaces, .. }) = &welcome
        else {
            panic!("期待 Welcome");
        };
        assert_eq!(*wid, 100);
        assert_eq!(extra_surfaces.len(), 1);
        assert_eq!(extra_surfaces[0].surface, 901);

        // 第二表面（wid=101）的帧路由到 surface=901，而非主表面 900。
        let frame = host
            .on_message(ProtocolMsg::Frame(FrameMsg::FrameReady {
                wid: 101,
                frame_id: 1,
                slot: 0,
                damage: None,
                revision: 1,
                payload: Default::default(),
            }))
            .unwrap();
        match &frame[0] {
            HostAction::ComposeFrame { surface, wid, .. } => {
                assert_eq!(*surface, 901, "按 wid 路由到第二表面");
                assert_eq!(*wid, 101);
            }
            other => panic!("期待 ComposeFrame，得到 {other:?}"),
        }
        // 主表面（wid=100）帧仍路由 900。
        let frame = host
            .on_message(ProtocolMsg::Frame(FrameMsg::FrameReady {
                wid: 100,
                frame_id: 2,
                slot: 0,
                damage: None,
                revision: 2,
                payload: Default::default(),
            }))
            .unwrap();
        match &frame[0] {
            HostAction::ComposeFrame { surface, .. } => assert_eq!(*surface, 900),
            other => panic!("期待 ComposeFrame，得到 {other:?}"),
        }

        // 旧端线 Hello（无尾段）→ 空 surfaces 声明（既有行为零变化）。
        let mut host2 = HostEndpoint::listen();
        let legacy = ProtocolMsg::Handshake(HandshakeMsg::Hello {
            version: super::super::PROTOCOL_VERSION,
            app_name: "counter".into(),
            title: "c".into(),
            icon: None,
            width: 10.0,
            height: 10.0,
            fonts: vec![],
            surfaces: Vec::new(),
        });
        match host2.on_message(legacy).unwrap()[0] {
            HostAction::ResolveAndAttach { ref surfaces, .. } => assert!(surfaces.is_empty()),
            ref other => panic!("期待 ResolveAndAttach，得到 {other:?}"),
        }
    }
}

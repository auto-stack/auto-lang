// Plan 386 Stage 1 —— 桌面协议五通道消息结构（后端中立：无 iced 依赖，
// 几何/颜色用自有 `WRect`/`Rgba8`；宿主侧适配在 `host`）。
//
// 每个 enum 一条通道；消息 tag = 变体在 match 中的显式编号（线格式冻结，
// 只许追加不许改义）。`ProtocolMsg` 是过线单元：信封（`codec`）+ 通道 +
// 载荷。Plan 413 §7 三点落位：IME 三变体在 `InputMsg`；字体注册
// `FontBlob` 在 `HandshakeMsg::Hello`；按行缓存失效 `CacheControl` 在
// `FrameMsg`。

use super::codec::*;

/// 线格式矩形（宿主窗坐标或 widget 本地坐标，按消息语义标注）。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl WRect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn encode(&self, out: &mut Vec<u8>) {
        put_f32(out, self.x);
        put_f32(out, self.y);
        put_f32(out, self.w);
        put_f32(out, self.h);
    }

    pub fn decode(r: &mut Reader<'_>) -> Result<Self, CodecError> {
        Ok(Self { x: r.f32()?, y: r.f32()?, w: r.f32()?, h: r.f32()? })
    }
}

/// 8bit RGBA（与 `code_editor::theme::Rgba` 同域，adpater 侧互转）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rgba8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba8 {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn encode(&self, out: &mut Vec<u8>) {
        put_u8(out, self.r);
        put_u8(out, self.g);
        put_u8(out, self.b);
        put_u8(out, self.a);
    }

    pub fn decode(r: &mut Reader<'_>) -> Result<Self, CodecError> {
        Ok(Self { r: r.u8()?, g: r.u8()?, b: r.u8()?, a: r.u8()? })
    }
}

/// 帧载荷 v1：最小显示列表（quad + text run，`EditorDrawList` 同型 lowering；
/// 待澄清事项①——全量 VTree→RenderCommand lowering 归 Stage 2，载荷种类
/// tag 预留扩展位）。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DrawList {
    /// 清屏色（None = 沿用宿主底色）。
    pub clear: Option<Rgba8>,
    /// 绘制序（先到先画，后画盖前）。
    pub ops: Vec<DrawOp>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DrawOp {
    /// 实心矩形（quad；widget 本地坐标）。
    Quad { rect: WRect, color: Rgba8 },
    /// 单行文本 run（左上角定位；shaping 留宿主——413 §7 约束同款）。
    Text { x: f32, y: f32, size: f32, line_height: f32, color: Rgba8, text: String },
}

impl DrawList {
    pub fn encode(&self, out: &mut Vec<u8>) {
        put_u8(out, 1); // 载荷种类 tag：1 = DrawList（扩展位）
        match self.clear {
            Some(c) => {
                put_bool(out, true);
                c.encode(out);
            }
            None => put_bool(out, false),
        }
        put_u32(out, self.ops.len() as u32);
        for op in &self.ops {
            match op {
                DrawOp::Quad { rect, color } => {
                    put_u8(out, 1);
                    rect.encode(out);
                    color.encode(out);
                }
                DrawOp::Text { x, y, size, line_height, color, text } => {
                    put_u8(out, 2);
                    put_f32(out, *x);
                    put_f32(out, *y);
                    put_f32(out, *size);
                    put_f32(out, *line_height);
                    color.encode(out);
                    put_string(out, text);
                }
            }
        }
    }

    pub fn decode(r: &mut Reader<'_>) -> Result<Self, CodecError> {
        let kind = r.u8()?;
        if kind != 1 {
            return Err(CodecError::UnknownTag(kind));
        }
        let clear = if r.bool()? { Some(Rgba8::decode(r)?) } else { None };
        let n = r.u32()? as usize;
        let mut ops = Vec::with_capacity(n.min(1024));
        for _ in 0..n {
            match r.u8()? {
                1 => {
                    let rect = WRect::decode(r)?;
                    let color = Rgba8::decode(r)?;
                    ops.push(DrawOp::Quad { rect, color });
                }
                2 => {
                    let x = r.f32()?;
                    let y = r.f32()?;
                    let size = r.f32()?;
                    let line_height = r.f32()?;
                    let color = Rgba8::decode(r)?;
                    let text = r.string()?;
                    ops.push(DrawOp::Text { x, y, size, line_height, color, text });
                }
                tag => return Err(CodecError::UnknownTag(tag)),
            }
        }
        Ok(Self { clear, ops })
    }
}

/// App 自带字体上传（413 §7.2：分离模式下宿主 shaping 需要 app 的字体）。
#[derive(Debug, Clone, PartialEq)]
pub struct FontBlob {
    pub family: String,
    pub data: Vec<u8>,
}

// ---------------------------------------------------------------------------
// 通道 1：孵化/握手
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum HandshakeMsg {
    /// app→host。spawn/反向连接后的第一跳：上报身份 + 初始尺寸 + 字体。
    Hello {
        version: u16,
        app_name: String,
        title: String,
        icon: Option<Vec<u8>>,
        width: f32,
        height: f32,
        fonts: Vec<FontBlob>,
    },
    /// host→app。分配结果：AppId + 虚拟窗 Wid + surface 句柄 + 初始矩形。
    Welcome { app_id: u64, wid: u64, surface: u64, rect: WRect },
    /// app→host。握手完成确认（状态机 Active 的入场合）。
    Ready,
}

impl HandshakeMsg {
    const HELLO: u8 = 1;
    const WELCOME: u8 = 2;
    const READY: u8 = 3;

    pub fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::Hello { version, app_name, title, icon, width, height, fonts } => {
                put_u8(out, Self::HELLO);
                put_u16(out, *version);
                put_string(out, app_name);
                put_string(out, title);
                match icon {
                    Some(bytes) => {
                        put_bool(out, true);
                        put_bytes(out, bytes);
                    }
                    None => put_bool(out, false),
                }
                put_f32(out, *width);
                put_f32(out, *height);
                put_u32(out, fonts.len() as u32);
                for f in fonts {
                    put_string(out, &f.family);
                    put_bytes(out, &f.data);
                }
            }
            Self::Welcome { app_id, wid, surface, rect } => {
                put_u8(out, Self::WELCOME);
                put_u64(out, *app_id);
                put_u64(out, *wid);
                put_u64(out, *surface);
                rect.encode(out);
            }
            Self::Ready => put_u8(out, Self::READY),
        }
    }

    pub fn decode(r: &mut Reader<'_>) -> Result<Self, CodecError> {
        Ok(match r.u8()? {
            Self::HELLO => {
                let version = r.u16()?;
                let app_name = r.string()?;
                let title = r.string()?;
                let icon = if r.bool()? { Some(r.bytes()?) } else { None };
                let width = r.f32()?;
                let height = r.f32()?;
                let n = r.u32()? as usize;
                let mut fonts = Vec::with_capacity(n.min(64));
                for _ in 0..n {
                    let family = r.string()?;
                    let data = r.bytes()?;
                    fonts.push(FontBlob { family, data });
                }
                Self::Hello { version, app_name, title, icon, width, height, fonts }
            }
            Self::WELCOME => {
                let app_id = r.u64()?;
                let wid = r.u64()?;
                let surface = r.u64()?;
                let rect = WRect::decode(r)?;
                Self::Welcome { app_id, wid, surface, rect }
            }
            Self::READY => Self::Ready,
            tag => return Err(CodecError::UnknownTag(tag)),
        })
    }
}

// ---------------------------------------------------------------------------
// 通道 2：帧（共享缓冲模拟；`FrameReady` 是唯一 app→host 方向变体）
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum FrameMsg {
    /// host→app。缓冲槽分配/重分配（握手隐含一次 alloc(2)；Resize 后同型）。
    BufferAlloc { surface: u64, slots: u8, width: f32, height: f32 },
    /// host→app。回收全部槽（窗口关闭/独立出去）。
    BufferRelease { surface: u64 },
    /// host→app。虚拟窗尺寸变更（重协商缓冲）。
    Resize { surface: u64, width: f32, height: f32 },
    /// app→host。帧就绪：写入 `slot`，`damage` = 脏区（None = 全帧），
    /// `revision` = 内容单调版本（宿主/宿主侧缓存键，413 §7.3 同源）。
    FrameReady { wid: u64, frame_id: u64, slot: u8, damage: Option<WRect>, revision: u64, payload: DrawList },
    /// host→app。帧合成完毕，归还 `slot` 给 app 的空闲池（双缓冲轮转）。
    FrameAck { wid: u64, frame_id: u64, slot: u8 },
    /// app→host。缓存失效提示（键域由生产者定义；编辑器 = revision×fold
    /// 组合键，413 §7.3）。
    CacheControl { wid: u64, drop_keys: Vec<u64> },
}

impl FrameMsg {
    const BUFFER_ALLOC: u8 = 1;
    const BUFFER_RELEASE: u8 = 2;
    const RESIZE: u8 = 3;
    const FRAME_READY: u8 = 4;
    const FRAME_ACK: u8 = 5;
    const CACHE_CONTROL: u8 = 6;

    pub fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::BufferAlloc { surface, slots, width, height } => {
                put_u8(out, Self::BUFFER_ALLOC);
                put_u64(out, *surface);
                put_u8(out, *slots);
                put_f32(out, *width);
                put_f32(out, *height);
            }
            Self::BufferRelease { surface } => {
                put_u8(out, Self::BUFFER_RELEASE);
                put_u64(out, *surface);
            }
            Self::Resize { surface, width, height } => {
                put_u8(out, Self::RESIZE);
                put_u64(out, *surface);
                put_f32(out, *width);
                put_f32(out, *height);
            }
            Self::FrameReady { wid, frame_id, slot, damage, revision, payload } => {
                put_u8(out, Self::FRAME_READY);
                put_u64(out, *wid);
                put_u64(out, *frame_id);
                put_u8(out, *slot);
                match damage {
                    Some(d) => {
                        put_bool(out, true);
                        d.encode(out);
                    }
                    None => put_bool(out, false),
                }
                put_u64(out, *revision);
                payload.encode(out);
            }
            Self::FrameAck { wid, frame_id, slot } => {
                put_u8(out, Self::FRAME_ACK);
                put_u64(out, *wid);
                put_u64(out, *frame_id);
                put_u8(out, *slot);
            }
            Self::CacheControl { wid, drop_keys } => {
                put_u8(out, Self::CACHE_CONTROL);
                put_u64(out, *wid);
                put_u32(out, drop_keys.len() as u32);
                for k in drop_keys {
                    put_u64(out, *k);
                }
            }
        }
    }

    pub fn decode(r: &mut Reader<'_>) -> Result<Self, CodecError> {
        Ok(match r.u8()? {
            Self::BUFFER_ALLOC => {
                let surface = r.u64()?;
                let slots = r.u8()?;
                let width = r.f32()?;
                let height = r.f32()?;
                Self::BufferAlloc { surface, slots, width, height }
            }
            Self::BUFFER_RELEASE => Self::BufferRelease { surface: r.u64()? },
            Self::RESIZE => {
                let surface = r.u64()?;
                let width = r.f32()?;
                let height = r.f32()?;
                Self::Resize { surface, width, height }
            }
            Self::FRAME_READY => {
                let wid = r.u64()?;
                let frame_id = r.u64()?;
                let slot = r.u8()?;
                let damage = if r.bool()? { Some(WRect::decode(r)?) } else { None };
                let revision = r.u64()?;
                let payload = DrawList::decode(r)?;
                Self::FrameReady { wid, frame_id, slot, damage, revision, payload }
            }
            Self::FRAME_ACK => {
                let wid = r.u64()?;
                let frame_id = r.u64()?;
                let slot = r.u8()?;
                Self::FrameAck { wid, frame_id, slot }
            }
            Self::CACHE_CONTROL => {
                let wid = r.u64()?;
                let n = r.u32()? as usize;
                let mut drop_keys = Vec::with_capacity(n.min(1024));
                for _ in 0..n {
                    drop_keys.push(r.u64()?);
                }
                Self::CacheControl { wid, drop_keys }
            }
            tag => return Err(CodecError::UnknownTag(tag)),
        })
    }
}

// ---------------------------------------------------------------------------
// 通道 3：输入（host→app；(Wid, event) 编码 = E1 的进程间版）
// ---------------------------------------------------------------------------

/// 指针键位（线格式 u8：1 左 / 2 右 / 3 中）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left = 1,
    Right = 2,
    Middle = 3,
}

impl MouseButton {
    pub fn from_u8(v: u8) -> Result<Self, CodecError> {
        match v {
            1 => Ok(Self::Left),
            2 => Ok(Self::Right),
            3 => Ok(Self::Middle),
            other => Err(CodecError::UnknownTag(other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMsg {
    PointerMoved { wid: u64, x: f32, y: f32 },
    PointerPressed { wid: u64, button: MouseButton, x: f32, y: f32, modifiers: u8 },
    PointerReleased { wid: u64, button: MouseButton, x: f32, y: f32, modifiers: u8 },
    /// 物理键码（宿主映射前的原始码；语义映射留 app 侧工具层）。
    KeyPressed { wid: u64, key: u32, modifiers: u8 },
    KeyReleased { wid: u64, key: u32, modifiers: u8 },
    CharTyped { wid: u64, ch: char },
    Scroll { wid: u64, dx: f32, dy: f32 },
    /// 413 §7.1：preedit 组合串 + 光标矩形（候选窗定位）。
    ImePreedit { wid: u64, text: String, cursor: WRect },
    ImeCommit { wid: u64, text: String },
    ImeCancelled { wid: u64 },
}

impl InputMsg {
    pub fn wid(&self) -> u64 {
        match self {
            Self::PointerMoved { wid, .. }
            | Self::PointerPressed { wid, .. }
            | Self::PointerReleased { wid, .. }
            | Self::KeyPressed { wid, .. }
            | Self::KeyReleased { wid, .. }
            | Self::CharTyped { wid, .. }
            | Self::Scroll { wid, .. }
            | Self::ImePreedit { wid, .. }
            | Self::ImeCommit { wid, .. }
            | Self::ImeCancelled { wid } => *wid,
        }
    }

    pub fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::PointerMoved { wid, x, y } => {
                put_u8(out, 1);
                put_u64(out, *wid);
                put_f32(out, *x);
                put_f32(out, *y);
            }
            Self::PointerPressed { wid, button, x, y, modifiers }
            | Self::PointerReleased { wid, button, x, y, modifiers } => {
                put_u8(out, if matches!(self, Self::PointerPressed { .. }) { 2 } else { 3 });
                put_u64(out, *wid);
                put_u8(out, *button as u8);
                put_f32(out, *x);
                put_f32(out, *y);
                put_u8(out, *modifiers);
            }
            Self::KeyPressed { wid, key, modifiers } => {
                put_u8(out, 4);
                put_u64(out, *wid);
                put_u32(out, *key);
                put_u8(out, *modifiers);
            }
            Self::KeyReleased { wid, key, modifiers } => {
                put_u8(out, 5);
                put_u64(out, *wid);
                put_u32(out, *key);
                put_u8(out, *modifiers);
            }
            Self::CharTyped { wid, ch } => {
                put_u8(out, 6);
                put_u64(out, *wid);
                let mut buf = [0u8; 4];
                put_string(out, ch.encode_utf8(&mut buf));
            }
            Self::Scroll { wid, dx, dy } => {
                put_u8(out, 7);
                put_u64(out, *wid);
                put_f32(out, *dx);
                put_f32(out, *dy);
            }
            Self::ImePreedit { wid, text, cursor } => {
                put_u8(out, 8);
                put_u64(out, *wid);
                put_string(out, text);
                cursor.encode(out);
            }
            Self::ImeCommit { wid, text } => {
                put_u8(out, 9);
                put_u64(out, *wid);
                put_string(out, text);
            }
            Self::ImeCancelled { wid } => {
                put_u8(out, 10);
                put_u64(out, *wid);
            }
        }
    }

    pub fn decode(r: &mut Reader<'_>) -> Result<Self, CodecError> {
        Ok(match r.u8()? {
            1 => {
                let wid = r.u64()?;
                let x = r.f32()?;
                let y = r.f32()?;
                Self::PointerMoved { wid, x, y }
            }
            tag @ (2 | 3) => {
                let wid = r.u64()?;
                let button = MouseButton::from_u8(r.u8()?)?;
                let x = r.f32()?;
                let y = r.f32()?;
                let modifiers = r.u8()?;
                if tag == 2 {
                    Self::PointerPressed { wid, button, x, y, modifiers }
                } else {
                    Self::PointerReleased { wid, button, x, y, modifiers }
                }
            }
            4 => {
                let wid = r.u64()?;
                let key = r.u32()?;
                let modifiers = r.u8()?;
                Self::KeyPressed { wid, key, modifiers }
            }
            5 => {
                let wid = r.u64()?;
                let key = r.u32()?;
                let modifiers = r.u8()?;
                Self::KeyReleased { wid, key, modifiers }
            }
            6 => {
                let wid = r.u64()?;
                let text = r.string()?;
                let mut chars = text.chars();
                let ch = chars.next().ok_or(CodecError::BadUtf8)?;
                Self::CharTyped { wid, ch }
            }
            7 => {
                let wid = r.u64()?;
                let dx = r.f32()?;
                let dy = r.f32()?;
                Self::Scroll { wid, dx, dy }
            }
            8 => {
                let wid = r.u64()?;
                let text = r.string()?;
                let cursor = WRect::decode(r)?;
                Self::ImePreedit { wid, text, cursor }
            }
            9 => {
                let wid = r.u64()?;
                let text = r.string()?;
                Self::ImeCommit { wid, text }
            }
            10 => Self::ImeCancelled { wid: r.u64()? },
            tag => return Err(CodecError::UnknownTag(tag)),
        })
    }
}

// ---------------------------------------------------------------------------
// 通道 4：控制（生命周期双向 + DesktopBus 跨进程载荷）
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum ControlMsg {
    /// host→app。请求退出（app 侧收尾后回 `ExitRequest` 确认）。
    Close { wid: u64 },
    /// host→app。焦点变化。
    Focus { wid: u64, focused: bool },
    /// host→app。虚拟窗 resize（帧通道 `Resize` 的生命周期孪生通知）。
    Resize { wid: u64, width: f32, height: f32 },
    /// app→host。标题变更（虚拟窗 chrome 同步）。
    TitleChanged { wid: u64, title: String },
    /// app→host。通知（桌面通知中心的最小载荷）。
    Notify { wid: u64, summary: String, body: String },
    /// app→host。确认退出 / 主动请求退出（宿主随即回收虚拟窗，462 Close 语义）。
    ExitRequest { wid: u64 },
    /// app→host。DesktopBus 跨进程：载荷 = 既有 `DesktopCommand` 单记录
    /// 编码串（`launch\u{1F}<name>` 等——shell.at 写入侧格式，宿主
    /// `DesktopCommand::parse_records` 直解析）。
    DesktopBus { wid: u64, record: String },
}

impl ControlMsg {
    pub fn wid(&self) -> u64 {
        match self {
            Self::Close { wid }
            | Self::Focus { wid, .. }
            | Self::Resize { wid, .. }
            | Self::TitleChanged { wid, .. }
            | Self::Notify { wid, .. }
            | Self::ExitRequest { wid }
            | Self::DesktopBus { wid, .. } => *wid,
        }
    }

    pub fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::Close { wid } => {
                put_u8(out, 1);
                put_u64(out, *wid);
            }
            Self::Focus { wid, focused } => {
                put_u8(out, 2);
                put_u64(out, *wid);
                put_bool(out, *focused);
            }
            Self::Resize { wid, width, height } => {
                put_u8(out, 3);
                put_u64(out, *wid);
                put_f32(out, *width);
                put_f32(out, *height);
            }
            Self::TitleChanged { wid, title } => {
                put_u8(out, 4);
                put_u64(out, *wid);
                put_string(out, title);
            }
            Self::Notify { wid, summary, body } => {
                put_u8(out, 5);
                put_u64(out, *wid);
                put_string(out, summary);
                put_string(out, body);
            }
            Self::ExitRequest { wid } => {
                put_u8(out, 6);
                put_u64(out, *wid);
            }
            Self::DesktopBus { wid, record } => {
                put_u8(out, 7);
                put_u64(out, *wid);
                put_string(out, record);
            }
        }
    }

    pub fn decode(r: &mut Reader<'_>) -> Result<Self, CodecError> {
        Ok(match r.u8()? {
            1 => Self::Close { wid: r.u64()? },
            2 => {
                let wid = r.u64()?;
                let focused = r.bool()?;
                Self::Focus { wid, focused }
            }
            3 => {
                let wid = r.u64()?;
                let width = r.f32()?;
                let height = r.f32()?;
                Self::Resize { wid, width, height }
            }
            4 => {
                let wid = r.u64()?;
                let title = r.string()?;
                Self::TitleChanged { wid, title }
            }
            5 => {
                let wid = r.u64()?;
                let summary = r.string()?;
                let body = r.string()?;
                Self::Notify { wid, summary, body }
            }
            6 => Self::ExitRequest { wid: r.u64()? },
            7 => {
                let wid = r.u64()?;
                let record = r.string()?;
                Self::DesktopBus { wid, record }
            }
            tag => return Err(CodecError::UnknownTag(tag)),
        })
    }
}

// ---------------------------------------------------------------------------
// 通道 5：观测（MCP/DevTools per-app 端口、桌面代理的最小底座）
// ---------------------------------------------------------------------------

/// 日志级别（线格式 u8：1 debug / 2 info / 3 warn / 4 error）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl LogLevel {
    pub fn from_u8(v: u8) -> Result<Self, CodecError> {
        match v {
            1 => Ok(Self::Debug),
            2 => Ok(Self::Info),
            3 => Ok(Self::Warn),
            4 => Ok(Self::Error),
            other => Err(CodecError::UnknownTag(other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObserveMsg {
    /// host→app。接观测汇（sink 名 = Stage 2 的 per-app MCP 端口名）。
    Attach { wid: u64, sink: String },
    /// host→app。摘除。
    Detach { wid: u64 },
    /// app→host。日志。
    Log { wid: u64, level: LogLevel, message: String },
    /// app→host。指标。
    Metric { wid: u64, key: String, value: f64 },
}

impl ObserveMsg {
    pub fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::Attach { wid, sink } => {
                put_u8(out, 1);
                put_u64(out, *wid);
                put_string(out, sink);
            }
            Self::Detach { wid } => {
                put_u8(out, 2);
                put_u64(out, *wid);
            }
            Self::Log { wid, level, message } => {
                put_u8(out, 3);
                put_u64(out, *wid);
                put_u8(out, *level as u8);
                put_string(out, message);
            }
            Self::Metric { wid, key, value } => {
                put_u8(out, 4);
                put_u64(out, *wid);
                put_string(out, key);
                put_f64(out, *value);
            }
        }
    }

    pub fn decode(r: &mut Reader<'_>) -> Result<Self, CodecError> {
        Ok(match r.u8()? {
            1 => {
                let wid = r.u64()?;
                let sink = r.string()?;
                Self::Attach { wid, sink }
            }
            2 => Self::Detach { wid: r.u64()? },
            3 => {
                let wid = r.u64()?;
                let level = LogLevel::from_u8(r.u8()?)?;
                let message = r.string()?;
                Self::Log { wid, level, message }
            }
            4 => {
                let wid = r.u64()?;
                let key = r.string()?;
                let value = r.f64()?;
                Self::Metric { wid, key, value }
            }
            tag => return Err(CodecError::UnknownTag(tag)),
        })
    }
}

// ---------------------------------------------------------------------------
// 过线单元
// ---------------------------------------------------------------------------

/// 一次过线的完整消息：信封（通道 + 版本）+ 载荷。
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolMsg {
    Handshake(HandshakeMsg),
    Frame(FrameMsg),
    Input(InputMsg),
    Control(ControlMsg),
    Observe(ObserveMsg),
}

impl ProtocolMsg {
    /// 全量编码为过线字节（含信封）。
    pub fn encode(&self) -> Vec<u8> {
        let channel = match self {
            Self::Handshake(_) => Channel::Handshake,
            Self::Frame(_) => Channel::Frame,
            Self::Input(_) => Channel::Input,
            Self::Control(_) => Channel::Control,
            Self::Observe(_) => Channel::Observe,
        };
        let mut body = Vec::new();
        match self {
            Self::Handshake(m) => m.encode(&mut body),
            Self::Frame(m) => m.encode(&mut body),
            Self::Input(m) => m.encode(&mut body),
            Self::Control(m) => m.encode(&mut body),
            Self::Observe(m) => m.encode(&mut body),
        }
        encode_envelope(super::PROTOCOL_VERSION, channel, &body)
    }

    /// 从过线字节解码（版本不符拒收）。
    pub fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        let (channel, version, payload) = decode_envelope(bytes)?;
        if version != super::PROTOCOL_VERSION {
            return Err(CodecError::UnsupportedVersion(version));
        }
        let mut r = Reader::new(payload);
        let msg = match channel {
            Channel::Handshake => Self::Handshake(HandshakeMsg::decode(&mut r)?),
            Channel::Frame => Self::Frame(FrameMsg::decode(&mut r)?),
            Channel::Input => Self::Input(InputMsg::decode(&mut r)?),
            Channel::Control => Self::Control(ControlMsg::decode(&mut r)?),
            Channel::Observe => Self::Observe(ObserveMsg::decode(&mut r)?),
        };
        r.finish()?;
        Ok(msg)
    }

    /// 所在通道（端点按通道做方向校验）。
    pub fn channel(&self) -> Channel {
        match self {
            Self::Handshake(_) => Channel::Handshake,
            Self::Frame(_) => Channel::Frame,
            Self::Input(_) => Channel::Input,
            Self::Control(_) => Channel::Control,
            Self::Observe(_) => Channel::Observe,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(msg: ProtocolMsg) -> ProtocolMsg {
        let bytes = msg.encode();
        let back = ProtocolMsg::decode(&bytes).unwrap_or_else(|e| panic!("decode {msg:?}: {e:?}"));
        assert_eq!(back, msg, "round trip 恒等");
        back
    }

    #[test]
    fn handshake_channel_round_trip() {
        round_trip(ProtocolMsg::Handshake(HandshakeMsg::Hello {
            version: super::super::PROTOCOL_VERSION,
            app_name: "counter".into(),
            title: "计数器".into(),
            icon: Some(vec![0x89, b'P', b'N', b'G']),
            width: 480.0,
            height: 320.0,
            fonts: vec![
                FontBlob { family: "JetBrains Mono".into(), data: vec![1, 2, 3, 4] },
                FontBlob { family: "Sans".into(), data: Vec::new() },
            ],
        }));
        round_trip(ProtocolMsg::Handshake(HandshakeMsg::Welcome {
            app_id: 1,
            wid: 3,
            surface: 42,
            rect: WRect::new(16.0, 16.0, 480.0, 320.0),
        }));
        round_trip(ProtocolMsg::Handshake(HandshakeMsg::Ready));
    }

    #[test]
    fn frame_channel_round_trip() {
        round_trip(ProtocolMsg::Frame(FrameMsg::BufferAlloc {
            surface: 42,
            slots: 2,
            width: 480.0,
            height: 320.0,
        }));
        round_trip(ProtocolMsg::Frame(FrameMsg::BufferRelease { surface: 42 }));
        round_trip(ProtocolMsg::Frame(FrameMsg::Resize { surface: 42, width: 640.0, height: 400.0 }));
        round_trip(ProtocolMsg::Frame(FrameMsg::FrameReady {
            wid: 3,
            frame_id: 7,
            slot: 1,
            damage: Some(WRect::new(0.0, 0.0, 120.0, 36.0)),
            revision: 9,
            payload: DrawList {
                clear: Some(Rgba8::new(24, 24, 28, 255)),
                ops: vec![
                    DrawOp::Quad { rect: WRect::new(10.0, 10.0, 120.0, 36.0), color: Rgba8::new(48, 96, 200, 255) },
                    DrawOp::Text { x: 20.0, y: 18.0, size: 14.0, line_height: 20.0, color: Rgba8::new(255, 255, 255, 255), text: "count: 1".into() },
                ],
            },
        }));
        round_trip(ProtocolMsg::Frame(FrameMsg::FrameReady {
            wid: 3,
            frame_id: 8,
            slot: 0,
            damage: None,
            revision: 10,
            payload: DrawList::default(),
        }));
        round_trip(ProtocolMsg::Frame(FrameMsg::FrameAck { wid: 3, frame_id: 7, slot: 1 }));
        round_trip(ProtocolMsg::Frame(FrameMsg::CacheControl { wid: 3, drop_keys: vec![1, 2, 0xDEAD_BEEF] }));
    }

    #[test]
    fn input_channel_round_trip() {
        let msgs = vec![
            InputMsg::PointerMoved { wid: 3, x: 1.5, y: 2.5 },
            InputMsg::PointerPressed { wid: 3, button: MouseButton::Left, x: 10.0, y: 20.0, modifiers: 0b101 },
            InputMsg::PointerReleased { wid: 3, button: MouseButton::Middle, x: 10.0, y: 20.0, modifiers: 0 },
            InputMsg::KeyPressed { wid: 3, key: 0x1B, modifiers: 1 },
            InputMsg::KeyReleased { wid: 3, key: 0x1B, modifiers: 0 },
            InputMsg::CharTyped { wid: 3, ch: '漢' },
            InputMsg::Scroll { wid: 3, dx: 0.0, dy: -33.5 },
            InputMsg::ImePreedit { wid: 3, text: "ni hao".into(), cursor: WRect::new(5.0, 6.0, 1.0, 14.0) },
            InputMsg::ImeCommit { wid: 3, text: "你好".into() },
            InputMsg::ImeCancelled { wid: 3 },
        ];
        for m in msgs {
            let msg = ProtocolMsg::Input(m);
            assert_eq!(msg.channel(), Channel::Input);
            round_trip(msg);
        }
    }

    #[test]
    fn control_channel_round_trip() {
        let msgs = vec![
            ControlMsg::Close { wid: 3 },
            ControlMsg::Focus { wid: 3, focused: true },
            ControlMsg::Resize { wid: 3, width: 100.0, height: 80.0 },
            ControlMsg::TitleChanged { wid: 3, title: "新标题".into() },
            ControlMsg::Notify { wid: 3, summary: "编译完成".into(), body: "0 warnings".into() },
            ControlMsg::ExitRequest { wid: 3 },
            ControlMsg::DesktopBus { wid: 3, record: "launch\u{1f}counter".into() },
        ];
        for m in msgs {
            assert_eq!(m.wid(), 3, "wid 提取器");
            round_trip(ProtocolMsg::Control(m));
        }
    }

    #[test]
    fn observe_channel_round_trip() {
        let msgs = vec![
            ObserveMsg::Attach { wid: 3, sink: "mcp://desktop/app-3".into() },
            ObserveMsg::Detach { wid: 3 },
            ObserveMsg::Log { wid: 3, level: LogLevel::Warn, message: "慢帧 22ms".into() },
            ObserveMsg::Metric { wid: 3, key: "frame_ms".into(), value: 16.6 },
        ];
        for m in msgs {
            round_trip(ProtocolMsg::Observe(m));
        }
    }

    #[test]
    fn per_channel_golden_bytes() {
        // 每通道一条 golden：信封头 + 载荷的线格式冻结锚点。
        // Handshake::Ready —— APDL|0100|01|00|01000000|03
        let bytes = ProtocolMsg::Handshake(HandshakeMsg::Ready).encode();
        assert_eq!(
            bytes,
            vec![b'A', b'P', b'D', b'L', 1, 0, 1, 0, 1, 0, 0, 0, 3]
        );
        // Input::ImeCancelled{wid:1} —— APDL|0100|03|00|09000000|0A|0100000000000000
        let bytes = ProtocolMsg::Input(InputMsg::ImeCancelled { wid: 1 }).encode();
        assert_eq!(
            bytes,
            vec![
                b'A', b'P', b'D', b'L', 1, 0, 3, 0, 9, 0, 0, 0, 10, 1, 0, 0, 0, 0, 0, 0, 0
            ]
        );
        // Control::Close{wid:2} —— APDL|0100|04|00|09000000|01|02…
        let bytes = ProtocolMsg::Control(ControlMsg::Close { wid: 2 }).encode();
        assert_eq!(
            bytes,
            vec![
                b'A', b'P', b'D', b'L', 1, 0, 4, 0, 9, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0
            ]
        );
        // Observe::Detach{wid:5} —— APDL|0100|05|00|09000000|02|05…
        let bytes = ProtocolMsg::Observe(ObserveMsg::Detach { wid: 5 }).encode();
        assert_eq!(
            bytes,
            vec![
                b'A', b'P', b'D', b'L', 1, 0, 5, 0, 9, 0, 0, 0, 2, 5, 0, 0, 0, 0, 0, 0, 0
            ]
        );
        // Frame::FrameAck{wid:1,frame_id:2,slot:1} —— APDL|0100|02|00|12000000|05|01…|02…|01
        let bytes =
            ProtocolMsg::Frame(FrameMsg::FrameAck { wid: 1, frame_id: 2, slot: 1 }).encode();
        assert_eq!(
            bytes,
            vec![
                b'A', b'P', b'D', b'L', 1, 0, 2, 0, 18, 0, 0, 0, 5, 1, 0, 0, 0, 0, 0, 0, 0, 2,
                0, 0, 0, 0, 0, 0, 0, 1
            ]
        );
    }

    #[test]
    fn rejects_unknown_tags_and_corruption() {
        // 未知消息 tag（各通道载荷首字节越界）。
        for (ch, mut bytes) in [
            (Channel::Handshake, ProtocolMsg::Handshake(HandshakeMsg::Ready).encode()),
            (Channel::Frame, ProtocolMsg::Frame(FrameMsg::BufferRelease { surface: 1 }).encode()),
            (Channel::Input, ProtocolMsg::Input(InputMsg::ImeCancelled { wid: 1 }).encode()),
            (Channel::Control, ProtocolMsg::Control(ControlMsg::Close { wid: 1 }).encode()),
            (Channel::Observe, ProtocolMsg::Observe(ObserveMsg::Detach { wid: 1 }).encode()),
        ] {
            bytes[12] = 0xEE; // 载荷首字节 = 消息 tag
            let (_, _, payload) = decode_envelope(&bytes).unwrap();
            assert_eq!(payload[0], 0xEE);
            match ch {
                Channel::Handshake => {
                    let mut r = Reader::new(payload);
                    assert_eq!(HandshakeMsg::decode(&mut r), Err(CodecError::UnknownTag(0xEE)))
                }
                Channel::Frame => {
                    let mut r = Reader::new(payload);
                    assert_eq!(FrameMsg::decode(&mut r), Err(CodecError::UnknownTag(0xEE)))
                }
                Channel::Input => {
                    let mut r = Reader::new(payload);
                    assert_eq!(InputMsg::decode(&mut r), Err(CodecError::UnknownTag(0xEE)))
                }
                Channel::Control => {
                    let mut r = Reader::new(payload);
                    assert_eq!(ControlMsg::decode(&mut r), Err(CodecError::UnknownTag(0xEE)))
                }
                Channel::Observe => {
                    let mut r = Reader::new(payload);
                    assert_eq!(ObserveMsg::decode(&mut r), Err(CodecError::UnknownTag(0xEE)))
                }
            }
        }
        // 未知 DrawOp tag：一个 Quad op 的载荷，第 7 字节（payload 内偏移
        // 6 = kind1 + clear1 + len4 之后）是 op tag。
        let mut bytes = ProtocolMsg::Frame(FrameMsg::FrameReady {
            wid: 1,
            frame_id: 1,
            slot: 0,
            damage: None,
            revision: 1,
            payload: DrawList {
                clear: None,
                ops: vec![DrawOp::Quad {
                    rect: WRect::new(0.0, 0.0, 1.0, 1.0),
                    color: Rgba8::new(0, 0, 0, 255),
                }],
            },
        })
        .encode();
        let (_, _, payload) = decode_envelope(&bytes).unwrap();
        // FrameReady 头 = tag1 + wid8 + frame_id8 + slot1 + damage1 + revision8
        // = 27 字节；DrawList 自此起：kind(1) + clear(1) + ops len(4) → op tag
        // 在 payload[27+6]=payload[33]。
        const DRAWLIST_AT: usize = 1 + 8 + 8 + 1 + 1 + 8;
        assert_eq!(payload[DRAWLIST_AT + 6], 1, "锚点：op tag 位置");
        bytes[12 + DRAWLIST_AT + 6] = 0x7F;
        let err = ProtocolMsg::decode(&bytes);
        assert!(matches!(err, Err(CodecError::UnknownTag(0x7F))), "got {err:?}");
        // 版本不符拒收。
        let mut bytes = ProtocolMsg::Control(ControlMsg::Close { wid: 1 }).encode();
        bytes[4] = 9;
        assert_eq!(
            ProtocolMsg::decode(&bytes),
            Err(CodecError::UnsupportedVersion(9))
        );
    }
}

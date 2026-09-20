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
    /// 带字重/斜体的文本 run（Plan 515 G2，tag 5 追加式；既有 `Text`
    /// 线格式冻结不动——weight = 400 且非斜体时投影器仍产 `Text`，旧端
    /// 远程消费面零破坏）。`weight` = CSS 字重刻度 u16（400 normal /
    /// 700 bold），`italic` = bool。
    TextStyled {
        x: f32,
        y: f32,
        size: f32,
        line_height: f32,
        color: Rgba8,
        weight: u16,
        italic: bool,
        text: String,
    },
    /// 压栈裁剪矩形（Plan 515 G1，tag 3 追加式）：宿主坐标空间，与当前
    /// 有效裁剪**取交集**后生效，作用于后续所有 op 直至配对 pop。栈语义
    /// 深度：线格式不设上限，v1 投影器产出 ≤2 层（嵌套 scrollable），
    /// 消费端须支持 ≥2 层（协议文档 §1.5）。编码端保证 push/pop 配对；
    /// 栅格端空栈 pop = no-op（解码端逐 op 无状态，不追踪栈）。
    Scissor { rect: WRect },
    /// 出栈最近一次 `Scissor`（tag 4 追加式）。
    ScissorPop,
    /// 图像引用（PLAN-028 图像通道，tag 6 追加式）：**src 引用 + 宿主侧
    /// 解析**——零位图字节过线，宿主按词汇表（本地文件 / `builtin:` /
    /// `data:` / `http(s)://` / `thumbnail://{wid}` 虚拟引用）解码缓存
    /// 后 `draw_image`；未解析 src = 宿主占位 + 观测行（not-yet 降级
    /// 纪律，禁静默错绘）。`fit` 最小枚举 v1 仅 Stretch（拉伸至 rect，
    /// 与占位尺寸盒同位）；filter/border_radius 不入 wire（宿主缺省
    /// Linear/方形，视觉与占位零差）——**op 字段定长不可尾部追加**
    /// （解码共享 Reader 无载荷尾判据），未来呈现参数 = 新 tag。
    Image { rect: WRect, src: String, fit: ImageFit },
}

/// 图像适配语义（PLAN-028 D1 定案：v1 最小集）。线格式 u8：1 Stretch。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageFit {
    /// 拉伸填充 rect（与占位尺寸盒语义同位）。
    #[default]
    Stretch,
}

impl ImageFit {
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Stretch => 1,
        }
    }

    pub fn from_u8(v: u8) -> Result<Self, CodecError> {
        match v {
            1 => Ok(Self::Stretch),
            other => Err(CodecError::UnknownTag(other)),
        }
    }
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
                DrawOp::TextStyled { x, y, size, line_height, color, weight, italic, text } => {
                    put_u8(out, 5);
                    put_f32(out, *x);
                    put_f32(out, *y);
                    put_f32(out, *size);
                    put_f32(out, *line_height);
                    color.encode(out);
                    put_u16(out, *weight);
                    put_bool(out, *italic);
                    put_string(out, text);
                }
                DrawOp::Scissor { rect } => {
                    put_u8(out, 3);
                    rect.encode(out);
                }
                DrawOp::ScissorPop => {
                    put_u8(out, 4);
                }
                DrawOp::Image { rect, src, fit } => {
                    put_u8(out, 6);
                    rect.encode(out);
                    put_string(out, src);
                    put_u8(out, fit.as_u8());
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
                5 => {
                    let x = r.f32()?;
                    let y = r.f32()?;
                    let size = r.f32()?;
                    let line_height = r.f32()?;
                    let color = Rgba8::decode(r)?;
                    let weight = r.u16()?;
                    let italic = r.bool()?;
                    let text = r.string()?;
                    ops.push(DrawOp::TextStyled { x, y, size, line_height, color, weight, italic, text });
                }
                3 => {
                    let rect = WRect::decode(r)?;
                    ops.push(DrawOp::Scissor { rect });
                }
                4 => ops.push(DrawOp::ScissorPop),
                6 => {
                    let rect = WRect::decode(r)?;
                    let src = r.string()?;
                    let fit = ImageFit::from_u8(r.u8()?)?;
                    ops.push(DrawOp::Image { rect, src, fit });
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

/// 帧载荷模式（v1.3 二态；`Welcome` 尾部协商位）。线格式 u8：1 Commands /
/// 2 Pixels。旧端载荷无此字段 → 解码缺省 Commands（追加式兼容）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FrameMode {
    /// 命令帧：shm 槽载 `[u32 len][DrawList 编码]`（v1.0 既有语义）。
    #[default]
    Commands,
    /// 像素帧：shm 槽载 `h × stride` RGBA 行序列（independent 臂自渲染）。
    Pixels,
}

impl FrameMode {
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Commands => 1,
            Self::Pixels => 2,
        }
    }

    pub fn from_u8(v: u8) -> Result<Self, CodecError> {
        match v {
            1 => Ok(Self::Commands),
            2 => Ok(Self::Pixels),
            other => Err(CodecError::UnknownTag(other)),
        }
    }
}

/// 像素帧格式（v1.3 定案：v1 仅 RGBA8 straight 非预乘，stride = w×4）。
/// 线格式 u8：1 Rgba8（扩展位）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelFormat {
    #[default]
    Rgba8,
}

impl PixelFormat {
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Rgba8 => 1,
        }
    }

    pub fn from_u8(v: u8) -> Result<Self, CodecError> {
        match v {
            1 => Ok(Self::Rgba8),
            other => Err(CodecError::UnknownTag(other)),
        }
    }
}

// ---------------------------------------------------------------------------
// 通道 1：孵化/握手
// ---------------------------------------------------------------------------

/// 表面 z 平面角色（v1.11 壳双表面声明；线格式 u8）。
pub mod surface_role {
    /// 常规 app 窗表面（缺省——旧端线等价）。
    pub const WINDOW: u8 = 0;
    /// 壳 background 表面：壁纸上、全部窗口下（全屏）。
    pub const BACKGROUND: u8 = 1;
    /// 壳 chrome 表面：全部窗口上（v1 = 任务栏带矩形，非全屏）。
    pub const CHROME: u8 = 2;
    /// PLAN-036 T-04：壳 overlay 表面（switcher/通知中心——全屏声明、
    /// 置顶伪窗、宿主 overlay 层贴放）。追加式档位（=3；既有值与 golden
    /// 零漂移）；面区分按 Hello 声明序（同 role 的 OVERLAY 依序映射
    /// switcher→notification_center——单 exe 内约定，wire 上 role 显式）。
    pub const OVERLAY: u8 = 3;

    pub fn name(v: u8) -> &'static str {
        match v {
            WINDOW => "window",
            BACKGROUND => "background",
            CHROME => "chrome",
            OVERLAY => "overlay",
            _ => "unknown",
        }
    }
}

/// 壳投影下行推的 face 寻址字节（v1.11；`ControlMsg::ShellProjectionPush`
/// 族）。值域对齐 `SHELL_MANIFEST` 五件（shell_projection.rs）。
pub mod shell_face {
    pub const SHELL: u8 = 1;
    pub const DESKTOP_SURFACE: u8 = 2;
    pub const SWITCHER: u8 = 3;
    pub const NOTIFICATION_CENTER: u8 = 4;
    pub const DASHBOARD: u8 = 5;

    pub fn name(v: u8) -> &'static str {
        match v {
            SHELL => "shell",
            DESKTOP_SURFACE => "desktop",
            SWITCHER => "switcher",
            NOTIFICATION_CENTER => "notification_center",
            DASHBOARD => "dashboard",
            _ => "unknown",
        }
    }
}

/// 客户端表面声明（v1.11：`Hello` 尾部追加，多表面协商）。单表面客户端
/// （既有全部 app）不发送该尾段 = 单 window 声明，零行为变化。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceDecl {
    /// `surface_role` 常量。
    pub role: u8,
    pub width: f32,
    pub height: f32,
}

impl SurfaceDecl {
    pub fn encode(&self, out: &mut Vec<u8>) {
        put_u8(out, self.role);
        put_f32(out, self.width);
        put_f32(out, self.height);
    }

    pub fn decode(r: &mut Reader<'_>) -> Result<Self, CodecError> {
        Ok(Self { role: r.u8()?, width: r.f32()?, height: r.f32()? })
    }
}

/// `Welcome` 尾部的逐表面分配结果（v1.11）。首表面仍走 Welcome 既有
/// 领头字段（wid/surface/rect）——旧端线无尾段即单表面。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WelcomeSurface {
    pub role: u8,
    pub wid: u64,
    pub surface: u64,
    pub rect: WRect,
}

impl WelcomeSurface {
    pub fn encode(&self, out: &mut Vec<u8>) {
        put_u8(out, self.role);
        put_u64(out, self.wid);
        put_u64(out, self.surface);
        self.rect.encode(out);
    }

    pub fn decode(r: &mut Reader<'_>) -> Result<Self, CodecError> {
        Ok(Self { role: r.u8()?, wid: r.u64()?, surface: r.u64()?, rect: WRect::decode(r)? })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HandshakeMsg {
    /// app→host。spawn/反向连接后的第一跳：上报身份 + 初始尺寸 + 字体。
    /// `surfaces`（v1.11 尾部追加）：多表面声明（壳双表面）；空 = 单
    /// window 表面（旧端线等价）。
    Hello {
        version: u16,
        app_name: String,
        title: String,
        icon: Option<Vec<u8>>,
        width: f32,
        height: f32,
        fonts: Vec<FontBlob>,
        surfaces: Vec<SurfaceDecl>,
    },
    /// host→app。分配结果：AppId + 虚拟窗 Wid + surface 句柄 + 初始矩形。
    /// `frame_mode`（v1.3 尾部追加）：该表面的帧载荷解释位——旧端载荷
    /// 缺此字段时解码缺省 Commands。`extra_surfaces`（v1.11 尾部追加）：
    /// 第二及以后表面（多表面客户端）；领头字段 = 首表面。
    Welcome {
        app_id: u64,
        wid: u64,
        surface: u64,
        rect: WRect,
        frame_mode: FrameMode,
        extra_surfaces: Vec<WelcomeSurface>,
    },
    /// app→host。握手完成确认（状态机 Active 的入场合）。
    Ready,
}

impl HandshakeMsg {
    const HELLO: u8 = 1;
    const WELCOME: u8 = 2;
    const READY: u8 = 3;

    pub fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::Hello { version, app_name, title, icon, width, height, fonts, surfaces } => {
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
                // v1.11 尾部追加：多表面声明（空则不写尾段——既有消息
                // 字节级不变；旧端线无此段 = 单 window）。
                if !surfaces.is_empty() {
                    put_u32(out, surfaces.len() as u32);
                    for s in surfaces {
                        s.encode(out);
                    }
                }
            }
            Self::Welcome { app_id, wid, surface, rect, frame_mode, extra_surfaces } => {
                put_u8(out, Self::WELCOME);
                put_u64(out, *app_id);
                put_u64(out, *wid);
                put_u64(out, *surface);
                rect.encode(out);
                put_u8(out, frame_mode.as_u8());
                // v1.11 尾部追加：第二及以后表面（空则不写尾段——既有
                // 消息字节级不变）。
                if !extra_surfaces.is_empty() {
                    put_u32(out, extra_surfaces.len() as u32);
                    for s in extra_surfaces {
                        s.encode(out);
                    }
                }
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
                // v1.11 尾部追加：旧端（v1.10 线）无此段 → 单 window 等价。
                let surfaces = if r.remaining() > 0 {
                    let n = r.u32()? as usize;
                    let mut surfaces = Vec::with_capacity(n.min(8));
                    for _ in 0..n {
                        surfaces.push(SurfaceDecl::decode(r)?);
                    }
                    surfaces
                } else {
                    Vec::new()
                };
                Self::Hello { version, app_name, title, icon, width, height, fonts, surfaces }
            }
            Self::WELCOME => {
                let app_id = r.u64()?;
                let wid = r.u64()?;
                let surface = r.u64()?;
                let rect = WRect::decode(r)?;
                // v1.3 尾部追加字段：旧端（v1.2 线）无此字节 → 缺省 Commands。
                let frame_mode = if r.remaining() > 0 {
                    FrameMode::from_u8(r.u8()?)?
                } else {
                    FrameMode::Commands
                };
                // v1.11 尾部追加：多表面（旧端线无此段 = 空）。
                let extra_surfaces = if r.remaining() > 0 {
                    let n = r.u32()? as usize;
                    let mut extras = Vec::with_capacity(n.min(8));
                    for _ in 0..n {
                        extras.push(WelcomeSurface::decode(r)?);
                    }
                    extras
                } else {
                    Vec::new()
                };
                Self::Welcome { app_id, wid, surface, rect, frame_mode, extra_surfaces }
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
    /// `shm` = 共享内存段名（S9：`autodesk-shm-<surface>` 约定；None = 纯
    /// 管道帧，Stage 1 loopback 形态）。
    /// `bm` = **PLAN-034 位图段尾追**（D3：`autodesk-shm-<pid>-<surface>-bm`
    /// 专用第二段——消息级尾追先例 = `shm` 字段本体 + Welcome frame_mode；
    /// None 不写字节 = 旧 golden 零漂移，decode 以 `remaining()` 条件读）。
    BufferAlloc { surface: u64, slots: u8, width: f32, height: f32, shm: Option<String>, bm: Option<BitmapBuffer> },
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
    /// app→host。帧就绪（**共享内存变体**，S9）：payload 在 `slot` 槽内
    /// （`[u32 len][DrawList 编码]`），管道上只过元数据——大帧不走管道。
    FrameReadyShared { wid: u64, frame_id: u64, slot: u8, damage: Option<WRect>, revision: u64, len: u32 },
    /// app→host。帧就绪（**像素帧变体**，v1.3 independent 臂）：RGBA 像素
    /// 在 `slot` 槽内（`h × stride` 行序列，格式 = `format`，straight 非
    /// 预乘），管道上只过元数据。槽载荷解释由 `Welcome.frame_mode` 协商。
    FrameReadyPixels {
        wid: u64,
        frame_id: u64,
        slot: u8,
        damage: Option<WRect>,
        revision: u64,
        /// 像素宽（像素数）。
        w: u32,
        /// 像素高（行数）。
        h: u32,
        /// 行字节距（Rgba8 = w×4；对齐留扩展）。
        stride: u32,
        /// 像素格式（v1 仅 Rgba8）。
        format: PixelFormat,
    },
    /// host→远程端点。交互区表（Plan 508 远程线；D3 表的过线形态）——
    /// 订阅兑现时下发一次：远程端本地命中判定（光标/点击寻址）用；
    /// 权威命中仍在 app 侧（坐标直传）。追加式变体：旧端不产不出。
    HitTable { wid: u64, hits: Vec<HitRegion> },
    /// app→host。**位图就绪**（PLAN-034 tag 10，D3：P028-D1 兑现）——
    /// app 生成的 RGBA 位图在位图段的 `slot` 槽内（`[u32 len][h×stride
    /// 行序列]`，straight 非预乘——FrameReadyPixels 同口径），管道上只过
    /// 元数据。`id` = app 侧位图标识（`bitmap://{id}` 词汇引用面）；同 id
    /// 重上传 = 宿主缓存即时翻新（无版本号，单写者时序由 Ack 纪律钉死）。
    /// 槽纪律镜像 FrameReadyShared：宿主读槽 → 入缓存 → `BitmapAck` 归还。
    BitmapReady {
        wid: u64,
        id: String,
        slot: u8,
        /// 像素宽（像素数）。
        w: u32,
        /// 像素高（行数）。
        h: u32,
        /// 行字节距（Rgba8 = w×4）。
        stride: u32,
        /// 槽内载荷字节长（含于 slot 头语义，显式镜像 FrameReadyShared）。
        len: u32,
    },
    /// host→app。**位图合成完毕**（tag 11）：归还 `slot` 给 app 的位图
    /// 空闲池（FrameAck 同纪律——app 复用槽必在 Ack 后 = 宿主已读完旧
    /// 载荷，同 id 覆盖时序由此钉死）。
    BitmapAck { wid: u64, slot: u8 },
}

/// `BufferAlloc.bm` 位图段声明（PLAN-034 D3）：专用第二段的段名/槽数/
/// 槽尺寸三元组——宿主按表面尺寸定档（slot_size = ceil(w)×ceil(h)×4+4，
/// v1 = 1× 逻辑分辨率），app 侧 [`super::shm::SharedFrameBuffer::open`]
/// 以此三元组开段（与主段 2×16384 约定档解耦）。
#[derive(Debug, Clone, PartialEq)]
pub struct BitmapBuffer {
    pub shm: String,
    pub slots: u8,
    pub slot_size: u32,
}

/// 交互区表条目（`HitTable` 载荷）：矩形 + 种类 + 动作串。
#[derive(Debug, Clone, PartialEq)]
pub struct HitRegion {
    pub rect: WRect,
    /// 1 = 按钮（action = handler 名）；2 = 输入框（action = 绑定字段）。
    pub kind: u8,
    pub action: String,
}

/// [`HitRegion::kind`] 常量。
pub const HIT_KIND_BUTTON: u8 = 1;
pub const HIT_KIND_INPUT: u8 = 2;

impl FrameMsg {
    const BUFFER_ALLOC: u8 = 1;
    const BUFFER_RELEASE: u8 = 2;
    const RESIZE: u8 = 3;
    const FRAME_READY: u8 = 4;
    const FRAME_ACK: u8 = 5;
    const CACHE_CONTROL: u8 = 6;
    const FRAME_READY_SHARED: u8 = 7;
    const FRAME_READY_PIXELS: u8 = 8;
    const HIT_TABLE: u8 = 9;
    const BITMAP_READY: u8 = 10;
    const BITMAP_ACK: u8 = 11;

    pub fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::BufferAlloc { surface, slots, width, height, shm, bm } => {
                put_u8(out, Self::BUFFER_ALLOC);
                put_u64(out, *surface);
                put_u8(out, *slots);
                put_f32(out, *width);
                put_f32(out, *height);
                match shm {
                    Some(name) => {
                        put_bool(out, true);
                        put_string(out, name);
                    }
                    None => put_bool(out, false),
                }
                // PLAN-034 D3：位图段尾追——None 不写字节（旧 golden
                // 零漂移；decode 侧 remaining() 条件读）。
                if let Some(bm) = bm {
                    put_bool(out, true);
                    put_string(out, &bm.shm);
                    put_u8(out, bm.slots);
                    put_u32(out, bm.slot_size);
                }
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
            Self::FrameReadyShared { wid, frame_id, slot, damage, revision, len } => {
                put_u8(out, Self::FRAME_READY_SHARED);
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
                put_u32(out, *len);
            }
            Self::FrameReadyPixels { wid, frame_id, slot, damage, revision, w, h, stride, format } => {
                put_u8(out, Self::FRAME_READY_PIXELS);
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
                put_u32(out, *w);
                put_u32(out, *h);
                put_u32(out, *stride);
                put_u8(out, format.as_u8());
            }
            Self::HitTable { wid, hits } => {
                put_u8(out, Self::HIT_TABLE);
                put_u64(out, *wid);
                put_u32(out, hits.len() as u32);
                for hit in hits {
                    hit.rect.encode(out);
                    put_u8(out, hit.kind);
                    put_string(out, &hit.action);
                }
            }
            Self::BitmapReady { wid, id, slot, w, h, stride, len } => {
                put_u8(out, Self::BITMAP_READY);
                put_u64(out, *wid);
                put_string(out, id);
                put_u8(out, *slot);
                put_u32(out, *w);
                put_u32(out, *h);
                put_u32(out, *stride);
                put_u32(out, *len);
            }
            Self::BitmapAck { wid, slot } => {
                put_u8(out, Self::BITMAP_ACK);
                put_u64(out, *wid);
                put_u8(out, *slot);
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
                let shm = if r.bool()? { Some(r.string()?) } else { None };
                // PLAN-034 D3：位图段尾追——remaining 条件读（Welcome
                // frame_mode 同律；旧端载荷无尾部 = None 缺省语义）。
                let bm = if r.remaining() > 0 && r.bool()? {
                    Some(BitmapBuffer {
                        shm: r.string()?,
                        slots: r.u8()?,
                        slot_size: r.u32()?,
                    })
                } else {
                    None
                };
                Self::BufferAlloc { surface, slots, width, height, shm, bm }
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
            Self::FRAME_READY_SHARED => {
                let wid = r.u64()?;
                let frame_id = r.u64()?;
                let slot = r.u8()?;
                let damage = if r.bool()? { Some(WRect::decode(r)?) } else { None };
                let revision = r.u64()?;
                let len = r.u32()?;
                Self::FrameReadyShared { wid, frame_id, slot, damage, revision, len }
            }
            Self::FRAME_READY_PIXELS => {
                let wid = r.u64()?;
                let frame_id = r.u64()?;
                let slot = r.u8()?;
                let damage = if r.bool()? { Some(WRect::decode(r)?) } else { None };
                let revision = r.u64()?;
                let w = r.u32()?;
                let h = r.u32()?;
                let stride = r.u32()?;
                let format = PixelFormat::from_u8(r.u8()?)?;
                Self::FrameReadyPixels { wid, frame_id, slot, damage, revision, w, h, stride, format }
            }
            Self::HIT_TABLE => {
                let wid = r.u64()?;
                let n = r.u32()? as usize;
                let mut hits = Vec::with_capacity(n.min(1024));
                for _ in 0..n {
                    let rect = WRect::decode(r)?;
                    let kind = r.u8()?;
                    let action = r.string()?;
                    hits.push(HitRegion { rect, kind, action });
                }
                Self::HitTable { wid, hits }
            }
            Self::BITMAP_READY => {
                let wid = r.u64()?;
                let id = r.string()?;
                let slot = r.u8()?;
                let w = r.u32()?;
                let h = r.u32()?;
                let stride = r.u32()?;
                let len = r.u32()?;
                Self::BitmapReady { wid, id, slot, w, h, stride, len }
            }
            Self::BITMAP_ACK => {
                let wid = r.u64()?;
                let slot = r.u8()?;
                Self::BitmapAck { wid, slot }
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
    /// host→app。L2"独立出去"：路线 B 客户端进程收到后切自管表面
    /// （自开 OS 窗/独立渲染循环），**VM 状态不动**；回 `L2Detached`
    /// 确认后宿主回收虚拟窗（autoshell §7.1 L2）。
    L2Detach { wid: u64 },
    /// app→host。`L2Detach` 的确认（宿主随即 ReclaimWindow）。
    L2Detached { wid: u64 },
    /// app→host。L2"进入 AutoDesk"：Standalone 的 app 请求重挂；
    /// 宿主按孵化处理（新 wid+surface），app 会话状态连续
    /// （revision 不归零 = 状态未动的协议级证据）。
    L2AttachRequest { wid: u64 },
    /// host→app。L3 v2a 快照迁移（Plan 480 S9）：融合态 App 的 AutoVM
    /// 状态快照注入恢复。载荷编码 = client_runtime 的 encode_state_snapshot
    /// （revision + 原始状态字段）。
    StateSnapshot { wid: u64, payload: Vec<u8> },
    /// host→app（v1.11 壳投影下行推）。face = `shell_face` 常量；payload =
    /// ShellProjection 家族 typed 载体 wire 编码（叶面保形——宿主侧
    /// `shell_projection::wire` 单源）。连接级寻址（非窗口），`wid()` = 0。
    ShellProjectionPush { face: u8, payload: Vec<u8> },
    /// host→app（v1.11 壳时钟）。分钟门数据（time = "HH:MM"、date =
    /// "M月D日 周X"）——独立脏帧通道，不入投影指纹门控组。
    ShellClockTick { face: u8, time: String, date: String },
    /// host→app（v1.11 壳光标事件）。事件级数据（设计上不入快照——
    /// 空白菜单坐标锚等）；节拍由宿主消费门控制（blank_menu/拖拽期 +
    /// tick 兜底）。坐标 = 宿主视口系。
    ShellCursorMove { face: u8, x: f32, y: f32 },
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
            | Self::DesktopBus { wid, .. }
            | Self::L2Detach { wid }
            | Self::L2Detached { wid }
            | Self::L2AttachRequest { wid }
            | Self::StateSnapshot { wid, .. } => *wid,
            // v1.11 壳投影族：连接级寻址（非窗口）——0 哨兵。
            Self::ShellProjectionPush { .. }
            | Self::ShellClockTick { .. }
            | Self::ShellCursorMove { .. } => 0,
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
            Self::L2Detach { wid } => {
                put_u8(out, 8);
                put_u64(out, *wid);
            }
            Self::L2Detached { wid } => {
                put_u8(out, 9);
                put_u64(out, *wid);
            }
            Self::L2AttachRequest { wid } => {
                put_u8(out, 10);
                put_u64(out, *wid);
            }
            Self::StateSnapshot { wid, payload } => {
                put_u8(out, 11);
                put_u64(out, *wid);
                put_bytes(out, payload);
            }
            Self::ShellProjectionPush { face, payload } => {
                put_u8(out, 12);
                put_u8(out, *face);
                put_bytes(out, payload);
            }
            Self::ShellClockTick { face, time, date } => {
                put_u8(out, 13);
                put_u8(out, *face);
                put_string(out, time);
                put_string(out, date);
            }
            Self::ShellCursorMove { face, x, y } => {
                put_u8(out, 14);
                put_u8(out, *face);
                put_f32(out, *x);
                put_f32(out, *y);
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
            8 => Self::L2Detach { wid: r.u64()? },
            9 => Self::L2Detached { wid: r.u64()? },
            10 => Self::L2AttachRequest { wid: r.u64()? },
            11 => {
                let wid = r.u64()?;
                let payload = r.bytes()?;
                Self::StateSnapshot { wid, payload }
            }
            12 => {
                let face = r.u8()?;
                let payload = r.bytes()?;
                Self::ShellProjectionPush { face, payload }
            }
            13 => {
                let face = r.u8()?;
                let time = r.string()?;
                let date = r.string()?;
                Self::ShellClockTick { face, time, date }
            }
            14 => {
                let face = r.u8()?;
                let x = r.f32()?;
                let y = r.f32()?;
                Self::ShellCursorMove { face, x, y }
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
            surfaces: Vec::new(),
        }));
        round_trip(ProtocolMsg::Handshake(HandshakeMsg::Hello {
            version: super::super::PROTOCOL_VERSION,
            app_name: "shell".into(),
            title: "shell".into(),
            icon: None,
            width: 1280.0,
            height: 800.0,
            fonts: Vec::new(),
            // v1.11 壳双表面声明（Hello 尾段）。
            surfaces: vec![
                SurfaceDecl { role: surface_role::BACKGROUND, width: 1280.0, height: 800.0 },
                SurfaceDecl { role: surface_role::CHROME, width: 1280.0, height: 48.0 },
            ],
        }));
        round_trip(ProtocolMsg::Handshake(HandshakeMsg::Welcome {
            app_id: 1,
            wid: 3,
            surface: 42,
            rect: WRect::new(16.0, 16.0, 480.0, 320.0),
            frame_mode: FrameMode::Commands,
            extra_surfaces: Vec::new(),
        }));
        round_trip(ProtocolMsg::Handshake(HandshakeMsg::Welcome {
            app_id: 2,
            wid: 4,
            surface: 43,
            rect: WRect::new(0.0, 0.0, 100.0, 80.0),
            frame_mode: FrameMode::Pixels,
            extra_surfaces: Vec::new(),
        }));
        round_trip(ProtocolMsg::Handshake(HandshakeMsg::Welcome {
            app_id: 9,
            wid: 100,
            surface: 900,
            rect: WRect::new(0.0, 0.0, 1280.0, 800.0),
            frame_mode: FrameMode::Commands,
            // v1.11 多表面 Welcome（尾段：chrome 面）。
            extra_surfaces: vec![WelcomeSurface {
                role: surface_role::CHROME,
                wid: 101,
                surface: 901,
                rect: WRect::new(0.0, 752.0, 1280.0, 48.0),
            }],
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
            shm: Some("autodesk-shm-42".into()),
            bm: None,
        }));
        round_trip(ProtocolMsg::Frame(FrameMsg::BufferAlloc {
            surface: 43,
            slots: 2,
            width: 100.0,
            height: 80.0,
            shm: None,
            bm: None,
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
        round_trip(ProtocolMsg::Frame(FrameMsg::FrameReadyShared {
            wid: 3,
            frame_id: 11,
            slot: 1,
            damage: Some(WRect::new(0.0, 0.0, 480.0, 320.0)),
            revision: 12,
            len: 4096,
        }));
        round_trip(ProtocolMsg::Frame(FrameMsg::FrameReadyPixels {
            wid: 3,
            frame_id: 12,
            slot: 0,
            damage: Some(WRect::new(0.0, 0.0, 480.0, 320.0)),
            revision: 13,
            w: 480,
            h: 320,
            stride: 1920,
            format: PixelFormat::Rgba8,
        }));
        round_trip(ProtocolMsg::Frame(FrameMsg::FrameReadyPixels {
            wid: 9,
            frame_id: 1,
            slot: 1,
            damage: None,
            revision: 1,
            w: 1,
            h: 1,
            stride: 4,
            format: PixelFormat::Rgba8,
        }));
    }

    /// PLAN-034 T-04：位图通道 wire——tag 10/11 round-trip 恒等 +
    /// BufferAlloc.bm 尾追 round-trip。
    #[test]
    fn p034_bitmap_wire_round_trip() {
        round_trip(ProtocolMsg::Frame(FrameMsg::BitmapReady {
            wid: 7,
            id: "1234-canvas-0".into(),
            slot: 1,
            w: 560,
            h: 360,
            stride: 2240,
            len: 806400,
        }));
        round_trip(ProtocolMsg::Frame(FrameMsg::BitmapReady {
            wid: 1,
            id: "a".into(),
            slot: 0,
            w: 1,
            h: 1,
            stride: 4,
            len: 4,
        }));
        round_trip(ProtocolMsg::Frame(FrameMsg::BitmapAck { wid: 7, slot: 1 }));
        round_trip(ProtocolMsg::Frame(FrameMsg::BufferAlloc {
            surface: 42,
            slots: 2,
            width: 480.0,
            height: 320.0,
            shm: Some("autodesk-shm-100-42".into()),
            bm: Some(BitmapBuffer {
                shm: "autodesk-shm-100-42-bm".into(),
                slots: 2,
                slot_size: 480 * 320 * 4 + 4,
            }),
        }));
    }

    /// PLAN-034 T-04（D3）：BufferAlloc.bm 尾追向后兼容——bm=None 编码
    /// 与旧线字节恒等（golden 零漂移）；旧线（无尾追）解码 bm=None；
    /// 新线尾追可解；未知 Frame tag（12 起仍未知）拒收维持。
    #[test]
    fn p034_bufferalloc_bm_tail_backward_compat() {
        // 旧线载荷（v1.14 线格式）：tag1 + surface + slots + w + h + shm。
        let legacy_payload = {
            let mut p = Vec::new();
            put_u8(&mut p, 1); // BUFFER_ALLOC
            put_u64(&mut p, 42);
            put_u8(&mut p, 2);
            put_f32(&mut p, 480.0);
            put_f32(&mut p, 320.0);
            put_bool(&mut p, true);
            put_string(&mut p, "autodesk-shm-100-42");
            p
        };
        // bm=None 的当前端编码 = 逐字节恒等（零漂移断言）。
        let current = ProtocolMsg::Frame(FrameMsg::BufferAlloc {
            surface: 42,
            slots: 2,
            width: 480.0,
            height: 320.0,
            shm: Some("autodesk-shm-100-42".into()),
            bm: None,
        })
        .encode();
        assert_eq!(
            &current[12..],
            &legacy_payload[..],
            "bm=None 编码与旧线字节恒等（I1 golden 零漂移）"
        );
        // 旧线可解：bm 缺省 None。
        let bytes = super::super::codec::encode_envelope(
            super::super::PROTOCOL_VERSION,
            super::super::codec::Channel::Frame,
            &legacy_payload,
        );
        assert_eq!(
            ProtocolMsg::decode(&bytes).expect("旧线 BufferAlloc 可解"),
            ProtocolMsg::Frame(FrameMsg::BufferAlloc {
                surface: 42,
                slots: 2,
                width: 480.0,
                height: 320.0,
                shm: Some("autodesk-shm-100-42".into()),
                bm: None,
            }),
            "旧端缺省 = bm None"
        );
        // 新线尾追：bool(true) + string + u8 + u32。
        let mut tail = Vec::new();
        put_bool(&mut tail, true);
        put_string(&mut tail, "autodesk-shm-100-42-bm");
        put_u8(&mut tail, 2);
        put_u32(&mut tail, 480 * 320 * 4 + 4);
        let mut new_bytes = bytes.clone();
        let at = 12 + legacy_payload.len();
        new_bytes.splice(at..at, tail.iter().copied());
        new_bytes[8..12].copy_from_slice(
            &((legacy_payload.len() as u32 + tail.len() as u32).to_le_bytes()),
        );
        assert_eq!(
            ProtocolMsg::decode(&new_bytes).expect("新线 BufferAlloc 可解"),
            ProtocolMsg::Frame(FrameMsg::BufferAlloc {
                surface: 42,
                slots: 2,
                width: 480.0,
                height: 320.0,
                shm: Some("autodesk-shm-100-42".into()),
                bm: Some(BitmapBuffer {
                    shm: "autodesk-shm-100-42-bm".into(),
                    slots: 2,
                    slot_size: 480 * 320 * 4 + 4,
                }),
            }),
            "尾追三元组可解"
        );
        // 未知 Frame tag（12）拒收维持（防线纪律）。
        let mut unknown = Vec::new();
        put_u8(&mut unknown, 12);
        let unknown_bytes = super::super::codec::encode_envelope(
            super::super::PROTOCOL_VERSION,
            super::super::codec::Channel::Frame,
            &unknown,
        );
        assert!(ProtocolMsg::decode(&unknown_bytes).is_err(), "tag 12 未知拒收");
    }

    /// v1.3 追加式兼容：旧端 Welcome 载荷（无 frame_mode 尾字节）解码
    /// 缺省 Commands；新端恒写尾字节。未知 FrameMode/PixelFormat tag 拒收。
    #[test]
    fn frame_mode_tail_field_backward_compat() {
        // 旧线 Welcome：手工构造无尾字节的载荷（v1.2 线格式）。
        let legacy_payload = {
            let mut p = Vec::new();
            put_u8(&mut p, 2); // WELCOME
            put_u64(&mut p, 1);
            put_u64(&mut p, 3);
            put_u64(&mut p, 42);
            WRect::new(16.0, 16.0, 480.0, 320.0).encode(&mut p);
            p
        };
        let bytes = super::super::codec::encode_envelope(
            super::super::PROTOCOL_VERSION,
            super::super::codec::Channel::Handshake,
            &legacy_payload,
        );
        let decoded = ProtocolMsg::decode(&bytes).expect("旧线 Welcome 可解");
        assert_eq!(
            decoded,
            ProtocolMsg::Handshake(HandshakeMsg::Welcome {
                app_id: 1,
                wid: 3,
                surface: 42,
                rect: WRect::new(16.0, 16.0, 480.0, 320.0),
                frame_mode: FrameMode::Commands,
                extra_surfaces: Vec::new(),
            }),
            "旧端缺省 = Commands"
        );

        // 新线 Welcome：载荷尾追加 frame_mode 字节 + 信封长度 +1。
        let mut new_bytes = bytes.clone();
        let tail = 12 + legacy_payload.len();
        new_bytes.splice(tail..tail, [2u8]);
        new_bytes[8..12].copy_from_slice(&((legacy_payload.len() as u32 + 1).to_le_bytes()));
        assert_eq!(
            ProtocolMsg::decode(&new_bytes).expect("新线 Welcome 可解"),
            ProtocolMsg::Handshake(HandshakeMsg::Welcome {
                app_id: 1,
                wid: 3,
                surface: 42,
                rect: WRect::new(16.0, 16.0, 480.0, 320.0),
                frame_mode: FrameMode::Pixels,
                extra_surfaces: Vec::new(),
            })
        );

        // 未知 FrameMode tag（尾字节 = 9）拒收。
        let mut bad = new_bytes.clone();
        let payload_len = {
            let l = u32::from_le_bytes(bad[8..12].try_into().unwrap()) as usize;
            l
        };
        bad[12 + payload_len - 1] = 9;
        assert!(matches!(
            ProtocolMsg::decode(&bad),
            Err(CodecError::UnknownTag(9))
        ));

        // 未知 PixelFormat tag：FrameReadyPixels 载荷末字节 = 0x7F。
        let mut bytes = ProtocolMsg::Frame(FrameMsg::FrameReadyPixels {
            wid: 1,
            frame_id: 1,
            slot: 0,
            damage: None,
            revision: 1,
            w: 2,
            h: 2,
            stride: 8,
            format: PixelFormat::Rgba8,
        })
        .encode();
        let payload_len = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        bytes[12 + payload_len - 1] = 0x7F;
        assert!(matches!(
            ProtocolMsg::decode(&bytes),
            Err(CodecError::UnknownTag(0x7F))
        ));
    }

    /// FrameReadyPixels golden bytes：线格式冻结锚点（tag 8 + 元数据序）。
    #[test]
    fn frame_ready_pixels_golden_bytes() {
        let bytes = ProtocolMsg::Frame(FrameMsg::FrameReadyPixels {
            wid: 1,
            frame_id: 2,
            slot: 1,
            damage: None,
            revision: 3,
            w: 4,
            h: 5,
            stride: 16,
            format: PixelFormat::Rgba8,
        })
        .encode();
        // 载荷 = tag8(1) + wid(8) + frame_id(8) + slot(1) + damage(1) +
        // revision(8) + w(4) + h(4) + stride(4) + format(1) = 40 字节。
        assert_eq!(u32::from_le_bytes(bytes[8..12].try_into().unwrap()), 40);
        let expect: Vec<u8> = [
            b'A', b'P', b'D', b'L', 1, 0, 2, 0, 40, 0, 0, 0, // 信封
            8, // tag: FrameReadyPixels
            1, 0, 0, 0, 0, 0, 0, 0, // wid
            2, 0, 0, 0, 0, 0, 0, 0, // frame_id
            1, // slot
            0, // damage: None
            3, 0, 0, 0, 0, 0, 0, 0, // revision
            4, 0, 0, 0, // w
            5, 0, 0, 0, // h
            16, 0, 0, 0, // stride
            1, // format: Rgba8
        ]
        .to_vec();
        assert_eq!(bytes, expect);
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
            ControlMsg::L2Detach { wid: 3 },
            ControlMsg::L2Detached { wid: 3 },
            ControlMsg::L2AttachRequest { wid: 3 },
        ];
        for m in msgs {
            assert_eq!(m.wid(), 3, "wid 提取器");
            round_trip(ProtocolMsg::Control(m));
        }
        // v1.11 壳投影族（连接级寻址——wid() = 0 哨兵）。
        let shell_msgs = vec![
            ControlMsg::ShellProjectionPush { face: shell_face::SHELL, payload: vec![1, 2, 3] },
            ControlMsg::ShellClockTick { face: shell_face::SHELL, time: "09:05".into(), date: "9月19日 周六".into() },
            ControlMsg::ShellCursorMove { face: shell_face::DESKTOP_SURFACE, x: 128.5, y: 300.0 },
        ];
        for m in shell_msgs {
            assert_eq!(m.wid(), 0, "壳投影族连接级寻址");
            round_trip(ProtocolMsg::Control(m));
        }
    }

    /// PLAN-030 T-02：v1.11 壳投影族 golden 字节冻结锚（tag 12/13/14，
    /// face 字节 + 载荷——结构漂移哨兵；载荷级，不含信封头）。
    #[test]
    fn shell_projection_golden_bytes() {
        let mut p = Vec::new();
        ControlMsg::ShellProjectionPush { face: shell_face::SHELL, payload: vec![0xde, 0xad] }.encode(&mut p);
        assert_eq!(p, vec![12, 1, 2, 0, 0, 0, 0xde, 0xad]);
        let mut p = Vec::new();
        ControlMsg::ShellClockTick { face: shell_face::SHELL, time: "09:05".into(), date: "周六".into() }.encode(&mut p);
        assert_eq!(p, vec![13, 1, 5, 0, 0, 0, b'0', b'9', b':', b'0', b'5', 6, 0, 0, 0, 0xe5, 0x91, 0xa8, 0xe5, 0x85, 0xad]);
        let mut p = Vec::new();
        ControlMsg::ShellCursorMove { face: shell_face::DESKTOP_SURFACE, x: 0.0, y: 0.0 }.encode(&mut p);
        assert_eq!(p, vec![14, 2, 0, 0, 0, 0, 0, 0, 0, 0]);
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

    /// Plan 515 G1 —— scissor 裁剪算子（tag 3/4 追加式）：栈语义
    /// push/pop round-trip + DrawList 直编 golden 字节锚点。
    #[test]
    fn scissor_ops_round_trip_and_golden() {
        // round trip：push → 内容 op → pop 完整帧（溢出内容在宿主被裁）。
        round_trip(ProtocolMsg::Frame(FrameMsg::FrameReady {
            wid: 3,
            frame_id: 21,
            slot: 0,
            damage: None,
            revision: 21,
            payload: DrawList {
                clear: Some(Rgba8::new(24, 24, 28, 255)),
                ops: vec![
                    DrawOp::Scissor { rect: WRect::new(8.0, 8.0, 200.0, 160.0) },
                    DrawOp::Quad {
                        rect: WRect::new(0.0, 0.0, 400.0, 400.0),
                        color: Rgba8::new(255, 0, 0, 255),
                    },
                    DrawOp::ScissorPop,
                ],
            },
        }));

        // golden：DrawList 直编字节（tag 3 = push + 4×f32；tag 4 = pop 单字节）。
        let list = DrawList {
            clear: None,
            ops: vec![
                DrawOp::Scissor { rect: WRect::new(1.0, 2.0, 3.0, 4.0) },
                DrawOp::ScissorPop,
            ],
        };
        let mut buf = Vec::new();
        list.encode(&mut buf);
        let mut expect: Vec<u8> = vec![1, 0, 2, 0, 0, 0]; // kind=1, clear=None, ops=2
        expect.push(3); // Scissor push tag
        expect.extend_from_slice(&1.0f32.to_le_bytes());
        expect.extend_from_slice(&2.0f32.to_le_bytes());
        expect.extend_from_slice(&3.0f32.to_le_bytes());
        expect.extend_from_slice(&4.0f32.to_le_bytes());
        expect.push(4); // ScissorPop tag
        assert_eq!(buf, expect, "scissor 线格式冻结锚点");
    }

    /// Plan 515 G2 —— typography 差分通道（tag 5 追加式）：TextStyled
    /// round-trip + golden 字节锚点（weight u16 CSS 刻度 + italic bool）。
    #[test]
    fn text_styled_round_trip_and_golden() {
        round_trip(ProtocolMsg::Frame(FrameMsg::FrameReady {
            wid: 3,
            frame_id: 22,
            slot: 0,
            damage: None,
            revision: 22,
            payload: DrawList {
                clear: None,
                ops: vec![
                    DrawOp::TextStyled {
                        x: 10.0,
                        y: 20.0,
                        size: 16.0,
                        line_height: 21.6,
                        color: Rgba8::new(220, 220, 220, 255),
                        weight: 700,
                        italic: false,
                        text: "bold text".into(),
                    },
                    DrawOp::TextStyled {
                        x: 10.0,
                        y: 42.0,
                        size: 14.0,
                        line_height: 18.9,
                        color: Rgba8::new(200, 200, 200, 255),
                        weight: 400,
                        italic: true,
                        text: "italic text".into(),
                    },
                ],
            },
        }));

        // golden：DrawList 直编（tag 5 + x/y/size/lh + rgba + weight u16 +
        // italic bool + str）。
        let list = DrawList {
            clear: None,
            ops: vec![DrawOp::TextStyled {
                x: 1.0,
                y: 2.0,
                size: 3.0,
                line_height: 4.0,
                color: Rgba8::new(5, 6, 7, 8),
                weight: 700,
                italic: true,
                text: "hi".into(),
            }],
        };
        let mut buf = Vec::new();
        list.encode(&mut buf);
        let mut expect: Vec<u8> = vec![1, 0, 1, 0, 0, 0, 5]; // kind, clear, len, tag
        expect.extend_from_slice(&1.0f32.to_le_bytes());
        expect.extend_from_slice(&2.0f32.to_le_bytes());
        expect.extend_from_slice(&3.0f32.to_le_bytes());
        expect.extend_from_slice(&4.0f32.to_le_bytes());
        expect.extend_from_slice(&[5, 6, 7, 8]); // color
        expect.extend_from_slice(&700u16.to_le_bytes()); // weight
        expect.push(1); // italic
        expect.extend_from_slice(&2u32.to_le_bytes()); // str len
        expect.extend_from_slice(b"hi");
        assert_eq!(buf, expect, "TextStyled 线格式冻结锚点");
    }

    /// PLAN-028 图像通道（tag 6 追加式）：Image op round-trip（src 多
    /// 形态 + 空 src 容错）+ golden 字节锚点 + 未知 fit 拒收。
    #[test]
    fn image_op_round_trip_and_golden() {
        // round trip：src 词汇多形态（本地文件/http/data:/thumbnail://）。
        for src in [
            "D:/pics/wall.png",
            "https://cn.cravatar.com/avatar/abc.png",
            "data:image/png;base64,iVBORw0KGgo=",
            "thumbnail://42",
        ] {
            round_trip(ProtocolMsg::Frame(FrameMsg::FrameReady {
                wid: 3,
                frame_id: 28,
                slot: 0,
                damage: None,
                revision: 28,
                payload: DrawList {
                    clear: None,
                    ops: vec![DrawOp::Image {
                        rect: WRect::new(10.0, 20.0, 80.0, 80.0),
                        src: src.into(),
                        fit: ImageFit::Stretch,
                    }],
                },
            }));
        }
        // 空 src 容错（View::image("") 投影容差——宿主按未解析降级）。
        round_trip(ProtocolMsg::Frame(FrameMsg::FrameReady {
            wid: 3,
            frame_id: 29,
            slot: 0,
            damage: None,
            revision: 29,
            payload: DrawList {
                clear: None,
                ops: vec![DrawOp::Image {
                    rect: WRect::new(0.0, 0.0, 1.0, 1.0),
                    src: String::new(),
                    fit: ImageFit::Stretch,
                }],
            },
        }));

        // golden：DrawList 直编（tag 6 + rect 4×f32 + str + fit u8）。
        let list = DrawList {
            clear: None,
            ops: vec![DrawOp::Image {
                rect: WRect::new(1.0, 2.0, 3.0, 4.0),
                src: "https://example.com/a.png".into(),
                fit: ImageFit::Stretch,
            }],
        };
        let mut buf = Vec::new();
        list.encode(&mut buf);
        let mut expect: Vec<u8> = vec![1, 0, 1, 0, 0, 0, 6]; // kind, clear, len, tag
        expect.extend_from_slice(&1.0f32.to_le_bytes());
        expect.extend_from_slice(&2.0f32.to_le_bytes());
        expect.extend_from_slice(&3.0f32.to_le_bytes());
        expect.extend_from_slice(&4.0f32.to_le_bytes());
        let src = b"https://example.com/a.png";
        expect.extend_from_slice(&(src.len() as u32).to_le_bytes());
        expect.extend_from_slice(src);
        expect.push(1); // fit: Stretch
        assert_eq!(buf, expect, "Image 线格式冻结锚点");

        // 未知 fit tag（fit 字节 = 9）拒收——与 FrameMode/PixelFormat 同纪律。
        let mut bytes = ProtocolMsg::Frame(FrameMsg::FrameReady {
            wid: 1,
            frame_id: 1,
            slot: 0,
            damage: None,
            revision: 1,
            payload: list,
        })
        .encode();
        let payload_len = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        // FrameReady 头 27 + DrawList（kind+clear+len=6；op = tag1+rect16+
        // str_len4+str25+fit1 = 47）= 80。
        assert_eq!(payload_len, 80, "载荷长锚点");
        *bytes.last_mut().unwrap() = 9; // 载荷末字节 = fit
        assert!(matches!(
            ProtocolMsg::decode(&bytes),
            Err(CodecError::UnknownTag(9))
        ));
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

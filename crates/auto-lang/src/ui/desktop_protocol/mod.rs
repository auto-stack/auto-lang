// Plan 386 Stage 1 —— AutoUI 桌面协议（Desktop Protocol）v1：进程外 App
// 与桌面的五通道接缝，先在同进程 loopback 内按协议编码走通（Design 25
// §7 = 施工图；协议正身 = Stage 2"两进程"的设计，本模块即其消息/编解码/
// 状态机单源）。
//
// 五通道（`docs/design/autoui/autoshell.md` §7 表）：
// | 通道   | 方向      | 内容                                                        |
// |--------|-----------|-------------------------------------------------------------|
// | 孵化/握手 | app→host→app | Hello(标题/图标/尺寸/字体注册) → Welcome(Wid+surface 句柄) → Ready |
// | 帧     | app→host  | FrameReady(共享缓冲槽 + DrawList 载荷 + damage) / FrameAck 归还槽 |
// | 输入   | host→app  | (Wid, event) 编码注入（指针/键盘/滚轮/IME 三变体，413 §7.1）   |
// | 控制   | 双向      | Close/Focus/Resize ↓；TitleChanged/Notify/ExitRequest/DesktopBus ↑ |
// | 观测   | 双向      | Attach/Detach ↓；Log/Metric ↑（MCP per-app 代理的最小底座）    |
//
// 分层（Stage 2 换 transport 时只动 loopback 层）：
// - `codec`   ：二进制信封（magic "APDL" + 版本 + 通道 + 长度）与 LE 原语。
// - `message` ：五通道消息结构 + 逐消息 encode/decode（后端中立，无 iced）。
// - `endpoint`: 双端状态机（App: Detached→Handshaking→Active→Closing；
//   Host: Listening→Active），非法迁移返回 [`ProtocolError`]。
// - `loopback`: 同进程字节管道（send 编码过线 / recv 解码）——Stage 1 的
//   "共享纹理模拟"；Stage 2 换命名管道/共享内存时签名不变。
// - `host`    ：宿主侧绑定——把端点动作落到真实 462 会话对象
//   （`DesktopSession`/`WmState`）与 [`SurfaceStore`] 双缓冲表面。
//
// 不变量（本 Stage 验收）：同一消息 encode→decode 恒等；状态机拒绝一切
// 非法迁移；loopback demo 的 App 行为与直挂（in-process 直调）无差。

#[cfg(feature = "ui-iced")]
pub mod codec;
#[cfg(feature = "ui-iced")]
pub mod demo;
#[cfg(feature = "ui-iced")]
pub mod editor_frame;
#[cfg(feature = "ui-iced")]
pub mod endpoint;
#[cfg(feature = "ui-iced")]
pub mod host;
#[cfg(feature = "ui-iced")]
pub mod loopback;
#[cfg(feature = "ui-iced")]
pub mod message;

/// 协议版本（信封头携带；不一致拒收——`CodecError::UnsupportedVersion`）。
/// v1 = 本计划 Stage 1 定稿（见 `docs/design/autoui/desktop-protocol-v1.md`）。
pub const PROTOCOL_VERSION: u16 = 1;

pub use codec::{CodecError, Channel};
pub use endpoint::{AppEndpoint, FrameSource, HostEndpoint, HostAction, HostState, ProtocolError};
pub use host::{ProtocolHost, SurfaceStore};
pub use loopback::{loopback_pair, LoopbackEnd};
pub use message::{
    ControlMsg, DrawList, DrawOp, FontBlob, FrameMsg, HandshakeMsg, InputMsg, ObserveMsg,
    ProtocolMsg, Rgba8, WRect,
};

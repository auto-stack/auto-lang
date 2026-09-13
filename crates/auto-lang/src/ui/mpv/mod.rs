//! VM/iced 端的**原生播放引擎**（PLAN-617，T-16 起）。
//!
//! ## 是什么
//!
//! 用 `libloading` 在**运行时**加载 `libmpv`，把它的解封装/解码/A-V 同步/音频输出/
//! HDR 色调映射/字幕全部白拿，我们只解决「帧怎么进 iced」。
//!
//! 为什么不是 ffmpeg FFI、为什么不是 GL 后端、为什么是 SW：
//! 见 [PLAN-617 §2.5](../../../../docs/plans/617-030-video-player-real-rebuild.md)
//! 与 [Design 30 §4](../../../../docs/design/autoui/030-video-player.md)（T-15 的
//! 四选一裁定与实测数字）。
//!
//! ## 三条不可动摇的边界
//!
//! 1. **没有构建期原生依赖**：本机没装 mpv 也能编译、能过 CI；运行时缺失即降级
//!    （AC-20）。`MPV_SPIKE`/CI 都**不**安装任何系统媒体包。
//! 2. **缺失是正常分支**：`MpvEngine::new()` 返回 `Err(MpvUnavailable::NoLibrary)`，
//!    不 panic、不黑屏——渲染层据此走今日的诚实占位。
//! 3. **销毁顺序钉死在类型里**：render context 必须先于 mpv core 释放
//!    （render.h:119 记为 UB），见 [`engine::MpvEngine`] 的 `Drop`。
//!
//! ## 布局
//!
//! - [`locale`] —— `LC_NUMERIC` 必须是 `"C"`（mpv 的 C 环境前提）
//! - [`loader`] —— 运行库解析序 + 符号表（无状态）
//! - [`frame`] —— 帧目标缓冲（把 mpv 的 64 字节对齐要求编码进类型）
//! - [`engine`] —— 句柄与 render context 的生命周期（T-16 的核心）
//! - [`contract`] —— §2.3 受控媒体契约在 mpv 侧的实现（T-18）
//! - [`channel`]/[`present`] —— 帧上屏通道与全屏 blit（T-17，feature `mpv-gpu`）

pub mod contract;
pub mod engine;
pub mod frame;
pub mod loader;
pub mod locale;

// T-17：帧 → GPU 纹理 的通道（需 wgpu，故挂 `mpv-gpu`）。
#[cfg(feature = "mpv-gpu")]
pub mod channel;
#[cfg(feature = "mpv-gpu")]
pub mod present;

pub use contract::{MediaContract, VideoContractDown, VideoContractEvent};
pub use engine::{MpvEngine, MpvEventInfo, MpvUnavailable};
pub use frame::FrameBuffer;
pub use loader::{MpvApi, MpvLoadError, MpvSymbols};

#[cfg(feature = "mpv-gpu")]
pub use channel::{FrameChannelStats, FrameOutcome, VideoFrameChannel, VideoLatestWins};
#[cfg(feature = "mpv-gpu")]
pub use present::VideoPresenter;

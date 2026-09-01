//! Plan 509 路线 B —— Smithay 合成器宿主（winit/nested 开发形态）。
//!
//! T1 裁定（`docs/plans/reports/509-smithay-route-verdict.md`）：宿主只管
//! 合成，不含 iced；shell 面（.at 资产）经生产渲染链产出像素帧，宿主以
//! 纹理上屏（Stage 1 静态首帧；live attach = Stage 2 增量传输）。
//!
//! ## 跨平台构建
//!
//! 真合成循环用 smithay（Linux）。非 Linux 目标本 crate 编译为 stub
//! （同 `host-libcosmic` 的 cfg 模式），Windows 主仓 dev 流零影响。
//!
//! ## Stage 1 形态
//!
//! - winit 嵌套后端（WSLg 验证路径，待澄清①）；
//! - 单全屏面：`--frame <png>` 指定首帧纹理（生产链产物），缺省纯色；
//! - `--frames <n>` 限帧自动退出（冒烟/取证）；默认跑到窗口关闭。

/// 真合成循环（Linux/Wayland）。
#[cfg(target_os = "linux")]
pub mod linux;

/// 非 Linux：stub（保 Windows 编译绿——T2 验收）。
#[cfg(not(target_os = "linux"))]
pub mod fallback;

/// 运行宿主合成循环。
///
/// - **Linux**：起 winit 嵌套后端，合成至窗口关闭（或 `max_frames` 到限）。
/// - **非 Linux（dev）**：返回错误说明 Linux-only（cfg 门控保编译）。
pub fn run_host(
    frame_png: Option<&std::path::Path>,
    max_frames: Option<u64>,
) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        linux::run_host(frame_png, max_frames)
    }
    #[cfg(not(target_os = "linux"))]
    {
        fallback::run_host(frame_png, max_frames)
    }
}

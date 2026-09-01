//! Plan 509 路线 B —— 最小 Smithay 合成循环（winit/nested 开发形态）。
//!
//! 会话 = winit 嵌套后端（WSLg 下开"显示器"窗）；合成 = 单全屏面
//! （Stage 1：`--frame` PNG 纹理或纯色清屏，每帧 bind→render→draw→
//! finish→submit）。Stage 2+ 挂 wayland_frontend（xdg 客户端）与桌面
//! 协议 live attach。
//!
//! smithay 0.7 渲染模型：`Renderer::render(framebuffer, size, transform)`
//! 产出 `Frame`，元素经 `RenderElement::draw` 自绘，`Frame::finish` 归
//! 还 SyncPoint 后 `submit` 换缓冲。

use std::path::Path;
use std::time::Duration;

use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::element::texture::{TextureBuffer, TextureRenderElement};
use smithay::backend::renderer::element::{Element as _, Kind, RenderElement};
use smithay::backend::renderer::gles::{GlesRenderer, GlesTexture};
use smithay::backend::renderer::{Color32F, Frame as _, Renderer as _};
use smithay::backend::winit::{self, WinitEvent};
use smithay::utils::{Rectangle, Transform};

/// 清屏色（无 --frame 时的"合成循环活着"信号；近桌面深色底）。
const CLEAR: [f32; 4] = [0.09, 0.09, 0.13, 1.0];

/// 宿主合成循环主入口。`frame_png` = 首帧纹理源（RGBA PNG）；`max_frames`
/// = 限帧自动退出（冒烟取证），None = 跑到窗口关闭。
pub fn run_host(frame_png: Option<&Path>, max_frames: Option<u64>) -> Result<(), String> {
    let (mut backend, mut event_loop) =
        winit::init::<GlesRenderer>().map_err(|e| format!("winit init: {e:?}"))?;
    eprintln!(
        "[auto-smithay-host] winit backend up, window {}x{}",
        backend.window_size().w,
        backend.window_size().h
    );

    // Stage-1 静态纹理（生产链 PNG）。PNG RGBA 字序 [R,G,B,A] 对应
    // Fourcc::Abgr8888（DRM fourcc 按 LE 字序反转命名）。
    let texture: Option<TextureBuffer<GlesTexture>> = match frame_png {
        Some(p) => {
            let (rgba, w, h) = load_rgba(p)?;
            let buf = TextureBuffer::<GlesTexture>::from_memory(
                backend.renderer(),
                &rgba,
                Fourcc::Abgr8888,
                (w as i32, h as i32),
                false,
                1,
                Transform::Normal,
                None,
            )
            .map_err(|e| format!("import {}: {e:?}", p.display()))?;
            Some(buf)
        }
        None => None,
    };
    eprintln!(
        "[auto-smithay-host] stage-1 texture {}",
        if texture.is_some() { "loaded" } else { "absent (clear-color form)" }
    );

    let mut close = false;
    let mut composed: u64 = 0;
    loop {
        event_loop.dispatch_new_events(|ev| {
            if matches!(ev, WinitEvent::CloseRequested) {
                close = true;
            }
        });
        if close {
            eprintln!("[auto-smithay-host] close requested");
            break;
        }

        let size = backend.window_size();
        let damage = [Rectangle::from_size(size)];
        let (renderer, mut fb) =
            backend.bind().map_err(|e| format!("bind: {e:?}"))?;
        let mut frame =
            renderer.render(&mut fb, size, Transform::Normal).map_err(|e| format!("render: {e:?}"))?;
        match &texture {
            Some(buffer) => {
                // 全屏铺放（Stage 1 不做缩放保真：纹理元素按窗口尺寸拉伸，
                // 首帧验收口径 = dock+背景可见）。
                let element = TextureRenderElement::from_texture_buffer(
                    (0.0, 0.0),
                    buffer,
                    None,
                    None,
                    Some((size.w, size.h).into()),
                    Kind::Unspecified,
                );
                let dst = element.geometry((1.0, 1.0).into());
                RenderElement::<GlesRenderer>::draw(
                    &element,
                    &mut frame,
                    element.src(),
                    dst,
                    &damage,
                    &[],
                )
                .map_err(|e| format!("texture draw: {e:?}"))?;
            }
            None => {
                frame
                    .draw_solid(
                        Rectangle::from_size(size),
                        &damage,
                        Color32F::from(CLEAR),
                    )
                    .map_err(|e| format!("clear draw: {e:?}"))?;
            }
        }
        // 同线程同 GL 上下文即时 submit——fence 隐式同步，显式丢弃。
        let _sync = frame.finish().map_err(|e| format!("finish: {e:?}"))?;
        drop(fb);
        backend.submit(None).map_err(|e| format!("submit: {e:?}"))?;
        composed += 1;
        if composed == 1 || composed % 60 == 0 {
            eprintln!("[auto-smithay-host] composed frame #{composed}");
        }

        if let Some(max) = max_frames {
            if composed >= max {
                eprintln!("[auto-smithay-host] frame budget {max} reached, exiting");
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(16));
    }
    Ok(())
}

/// PNG → RGBA 字节 + 尺寸（image crate 解码，宿主无 iced）。
fn load_rgba(path: &Path) -> Result<(Vec<u8>, u32, u32), String> {
    let img = image::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    Ok((rgba.into_raw(), w, h))
}

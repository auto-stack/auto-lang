//! Plan 497 T1 spike：裁剪式整窗快照缩略可行性 demo（**临时 spike**——
//! T1 定案后保留至计划收尾清理，非产品代码）。
//!
//! 验证链路：真 iced 窗口（2×2 四色块）→ `iced::window::screenshot` 整窗
//! RGBA（物理像素 + scale_factor）→ 按**逻辑矩形 × scale_factor** 裁剪中央
//! 窗口区 → box 降采样（长边 ≤ 256）→ PNG 留痕 → 中心像素色断言 → exit 0。
//!
//! 运行：`cargo run -p auto-lang --features ui-iced --example t1_thumb_spike`
//! 产物：`tests/screenshots/t1-spike-full.png`（整窗）+
//! `tests/screenshots/t1-spike-thumb.png`（裁剪降采样缩略——即"真缩略图"）。
//!
//! T1 三候选结论（详见 plan 497 待澄清③回写）：
//! - headless 复用：❌ HeadlessRenderer 为 no-op（无窗口/GPU/栅格化），
//!   产出像素需自写软光栅器——成本不可接受。
//! - overlay 离屏 target：❌ iced 0.14 无公开子树离屏渲染 API（compositor
//!   不对应用层暴露）；成本 = 侵入 iced runtime。
//! - 裁剪式整窗快照（本 demo）：✅ 复用 Plan 285 screenshot 通道 +
//!   Screenshot.scale_factor（iced 官方文档注记即支持 widget-bounds 裁剪）。

use iced::widget::{container, row};
use iced::{Color, Element, Length, Size, Subscription, Task, Theme};
use std::time::Duration;

/// 裁剪目标：窗口中央逻辑区（模拟 VWinState.rect —— 生产路径里是
/// 虚拟窗口在宿主窗内的逻辑矩形，这里用固定中央区验证同一数学）。
const CROP_RECT: iced::Rectangle = iced::Rectangle {
    x: 100.0,
    y: 75.0,
    width: 200.0,
    height: 150.0,
};

/// 降采样上限（长边）——计划详细设计 §1 的 ≤256。
const THUMB_MAX: u32 = 256;

#[derive(Debug, Clone)]
enum Msg {
    Tick,
    Shot(iced::window::Screenshot),
}

struct App {
    shots: usize,
}

fn main() -> iced::Result {
    iced::application(
        move || App::new(),
        update,
        view,
    )
    .title("t1 spike")
    .window_size(Size::new(400.0, 300.0))
    .theme(|_a: &App| Theme::Dark)
    .subscription(|_app: &App| {
        iced::time::every(Duration::from_millis(800)).map(|_| Msg::Tick)
    })
    .run()
}

impl App {
    fn new() -> (Self, Task<Msg>) {
        (Self { shots: 0 }, Task::none())
    }
}

fn update(app: &mut App, msg: Msg) -> Task<Msg> {
    match msg {
        // 首个 tick（窗口已稳定呈现）→ 触发整窗截图。
        Msg::Tick if app.shots == 0 => {
            app.shots += 1;
            iced::window::oldest().then(move |maybe_id| match maybe_id {
                Some(id) => iced::window::screenshot(id).map(Msg::Shot),
                None => Task::none(),
            })
        }
        Msg::Shot(ss) => {
            if let Err(err) = process(&ss) {
                eprintln!("[t1-spike] FAILED: {err}");
                std::process::exit(2);
            }
            std::process::exit(0);
        }
        _ => Task::none(),
    }
}

fn quad<'a>(c: Color) -> iced::widget::Container<'a, Msg, Theme, iced::Renderer> {
    container(iced::widget::text(""))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_t: &Theme| container::Style {
            background: Some(c.into()),
            ..Default::default()
        })
}

/// 四色块 2×2：红/绿上排，蓝/黄下排（裁剪区横跨四色中心 → 降采样后
/// 四角可断言各自色相）。
fn view(_app: &App) -> Element<'_, Msg> {
    let top = row![quad(Color::from_rgb8(0xE0, 0x20, 0x20)), quad(Color::from_rgb8(0x20, 0xC0, 0x20))];
    let bottom = row![quad(Color::from_rgb8(0x20, 0x40, 0xE0)), quad(Color::from_rgb8(0xE0, 0xD0, 0x20))];
    iced::widget::column![top, bottom].into()
}

/// spike 主流程：裁剪 → 降采样 → PNG 留痕 → 中心四域色相断言。
fn process(ss: &iced::window::Screenshot) -> Result<(), String> {
    let img_w = ss.size.width;
    let img_h = ss.size.height;
    eprintln!(
        "[t1-spike] screenshot {}x{} phys, scale_factor={}",
        img_w, img_h, ss.scale_factor
    );

    // 留痕 1：整窗原片。
    let full = image::RgbaImage::from_raw(img_w, img_h, ss.rgba.as_ref().to_vec())
        .ok_or("整窗 RGBA 尺寸不匹配")?;
    let dir = std::path::Path::new("tests/screenshots");
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    full.save(dir.join("t1-spike-full.png")).map_err(|e| e.to_string())?;

    // 核心 1：逻辑 rect × scale_factor → 物理像素裁剪（生产 = VWinState.rect）。
    let (cropped, cw, ch) =
        crop_physical(ss.rgba.as_ref(), img_w, img_h, CROP_RECT, ss.scale_factor)
            .ok_or("裁剪区越界/零尺寸")?;

    // 核心 2：box 降采样（长边 ≤ THUMB_MAX，等比）。
    let (thumb, tw, th) = downsample_box(&cropped, cw, ch, THUMB_MAX);
    eprintln!("[t1-spike] crop {cw}x{ch} -> thumb {tw}x{th}");

    // 留痕 2：真缩略图。
    let thumb_img = image::RgbaImage::from_raw(tw, th, thumb.clone())
        .ok_or("缩略 RGBA 尺寸不匹配")?;
    thumb_img.save(dir.join("t1-spike-thumb.png")).map_err(|e| e.to_string())?;

    // 断言：四象限中心色相（裁剪区中心即四色块交界——四象限各取内缩点，
    // 允许 box 混色 25% 容差）。
    let px = |x: u32, y: u32| {
        let i = ((y * tw + x) * 4) as usize;
        (thumb[i], thumb[i + 1], thumb[i + 2])
    };
    let assert_hue = |label: &str, got: (u8, u8, u8), dom: usize, want: u8| -> Result<(), String> {
        let v = match dom {
            0 => got.0,
            1 => got.1,
            _ => got.2,
        };
        let diff = v.abs_diff(want);
        if diff > 32 {
            Err(format!("{label} 主色分量 {v} vs 期望 {want} (got={got:?})"))
        } else {
            Ok(())
        }
    };
    let (qx, qy) = (tw / 4, th / 4);
    assert_hue("左上=红", px(qx, qy), 0, 0xE0)?;
    assert_hue("右上=绿", px(tw - qx, qy), 1, 0xC0)?;
    assert_hue("左下=蓝", px(qx, th - qy), 2, 0xE0)?;
    assert_hue("右下=黄", px(tw - qx, th - qy), 0, 0xE0)?;
    eprintln!("[t1-spike] PASS: 四象限色相断言绿，缩略图已留痕");
    Ok(())
}

/// 逻辑矩形 → 物理像素裁剪（行拷贝）。clamp 保证窗口贴边/越界安全。
fn crop_physical(
    rgba: &[u8],
    img_w: u32,
    img_h: u32,
    rect: iced::Rectangle,
    scale: f32,
) -> Option<(Vec<u8>, u32, u32)> {
    let f = |v: f32| v.max(0.0);
    let px = (f(rect.x) * scale).round() as i64;
    let py = (f(rect.y) * scale).round() as i64;
    let pw = (f(rect.width) * scale).round() as i64;
    let ph = (f(rect.height) * scale).round() as i64;
    if pw <= 0 || ph <= 0 || px >= img_w as i64 || py >= img_h as i64 {
        return None;
    }
    let x0 = px.max(0) as u32;
    let y0 = py.max(0) as u32;
    let x1 = (px + pw).clamp(0, img_w as i64) as u32;
    let y1 = (py + ph).clamp(0, img_h as i64) as u32;
    let w = x1 - x0;
    let h = y1 - y0;
    if w == 0 || h == 0 {
        return None;
    }
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for y in y0..y1 {
        let start = ((y * img_w + x0) * 4) as usize;
        out.extend_from_slice(&rgba[start..start + (w * 4) as usize]);
    }
    Some((out, w, h))
}

/// box 降采样：等比缩到长边 ≤ max，逐 box 均值（最近邻边界对齐）。
fn downsample_box(rgba: &[u8], w: u32, h: u32, max: u32) -> (Vec<u8>, u32, u32) {
    let long = w.max(h);
    if long <= max {
        return (rgba.to_vec(), w, h);
    }
    let ratio = max as f32 / long as f32;
    let tw = ((w as f32 * ratio).round() as u32).max(1);
    let th = ((h as f32 * ratio).round() as u32).max(1);
    let mut out = vec![0u8; (tw * th * 4) as usize];
    for ty in 0..th {
        let y0 = (ty as u64 * h as u64) / th as u64;
        let y1 = (((ty + 1) as u64 * h as u64) / th as u64).max(y0 + 1);
        for tx in 0..tw {
            let x0 = (tx as u64 * w as u64) / tw as u64;
            let x1 = (((tx + 1) as u64 * w as u64) / tw as u64).max(x0 + 1);
            let (mut r, mut g, mut b, mut a) = (0u64, 0u64, 0u64, 0u64);
            let mut n = 0u64;
            for y in y0..y1.min(h as u64) {
                for x in x0..x1.min(w as u64) {
                    let i = (((y as u32 * w + x as u32) * 4) as usize);
                    r += rgba[i] as u64;
                    g += rgba[i + 1] as u64;
                    b += rgba[i + 2] as u64;
                    a += rgba[i + 3] as u64;
                    n += 1;
                }
            }
            let o = ((ty * tw + tx) * 4) as usize;
            out[o] = (r / n) as u8;
            out[o + 1] = (g / n) as u8;
            out[o + 2] = (b / n) as u8;
            out[o + 3] = (a / n) as u8;
        }
    }
    (out, tw, th)
}

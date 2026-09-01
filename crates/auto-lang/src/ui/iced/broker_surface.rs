//! Plan 500 步骤 5 —— broker 表面两态合成（宿主渲染臂）。
//!
//! 虚拟窗客户区内容源：**broker client 的帧**（进程外 App 经协议上报）
//! 取代宿主本地重渲染（v1.2 的临时形态——host 侧 dynamic_view 同源重画，
//! child 帧仅消息级断言；本模块兑现"live-iced 消费"，host.rs 头注遗留点）。
//!
//! 两态（`Welcome.frame_mode` 协商）：
//! - **Commands（queue 臂）**：`DrawList` → canvas Program 降级——Quad =
//!   抗锯齿 fill、Text = `fill_text` 宿主侧 shaping（D1 定案 A：宿主
//!   iced 文本栈，零新依赖）、clear = 底色。damage v1.3 作重绘提示
//!   （每帧全量重建几何，正确性不受损；Cache 局部化归 Stage 5）。
//! - **Pixels（independent 臂）**：shm RGBA → `image::Handle::from_rgba`
//!   上传（497 快照同通道口径）→ Image 挂客户区。

use crate::ui::desktop_protocol::message::{DrawList, DrawOp, Rgba8};
use crate::ui::desktop_protocol::stage3::PixelsSurface;
use crate::ui::session::{DesktopMessage, DesktopSession, Wid};

fn to_color(c: Rgba8) -> iced::Color {
    iced::Color::from_rgba8(c.r, c.g, c.b, c.a as f32 / 255.0)
}

/// CSS 字重刻度（100..900）→ iced `Weight` 档（Plan 515 G2）。
fn css_weight_to_iced(w: u16) -> iced::font::Weight {
    match w {
        100 => iced::font::Weight::Thin,
        200 => iced::font::Weight::ExtraLight,
        300 => iced::font::Weight::Light,
        500 => iced::font::Weight::Medium,
        600 => iced::font::Weight::Semibold,
        700 => iced::font::Weight::Bold,
        800 => iced::font::Weight::ExtraBold,
        900 => iced::font::Weight::Black,
        _ => iced::font::Weight::Normal,
    }
}

/// DrawList → canvas 绘制程序（queue 臂栅格化：宿主 GPU 抗锯齿）。
struct DrawListPainter {
    list: DrawList,
}

impl iced::widget::canvas::Program<DesktopMessage> for DrawListPainter {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        use iced::widget::canvas::{Frame, Path, Text};
        let mut frame = Frame::new(renderer, bounds.size());
        // clear 底色：整面铺（None = 透明，窗体容器底透出）。
        if let Some(clear) = self.list.clear {
            frame.fill_rectangle(
                iced::Point::ORIGIN,
                bounds.size(),
                to_color(clear),
            );
        }
        paint_ops(&mut frame, &self.list.ops);
        let _ = Path::new(|_| {});
        vec![frame.into_geometry()]
    }
}

/// Plan 515 G1 —— scissor 栈栅格化：`Scissor` 起一段 `with_clip`（匹配
/// pop 之间的 op 裁剪到矩形内；嵌套 push 自然取交——draft/paste 的组合
/// 裁剪语义）。空栈 pop / 未闭合 push（编码端违约）宽容不炸：pop =
/// no-op，未闭合 = 裁到序列尾。
fn paint_ops(frame: &mut iced::widget::canvas::Frame, ops: &[DrawOp]) {
    use iced::widget::canvas::Text;
    let mut i = 0;
    while i < ops.len() {
        match &ops[i] {
            DrawOp::Scissor { rect } => {
                // 深度扫描找配对 pop（含嵌套层）。
                let mut depth = 1usize;
                let mut end = ops.len();
                for (j, op) in ops.iter().enumerate().take(ops.len()).skip(i + 1) {
                    match op {
                        DrawOp::Scissor { .. } => depth += 1,
                        DrawOp::ScissorPop => {
                            depth -= 1;
                            if depth == 0 {
                                end = j;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                let region = iced::Rectangle::new(
                    iced::Point::new(rect.x, rect.y),
                    iced::Size::new(rect.w.max(0.0), rect.h.max(0.0)),
                );
                frame.with_clip(region, |f| paint_ops(f, &ops[i + 1..end]));
                // 跳过配对 pop（未闭合时 end = ops.len()，循环自然收）。
                i = end + 1;
            }
            // 本层游离 pop（编码端违约）= no-op。
            DrawOp::ScissorPop => i += 1,
            DrawOp::Quad { rect, color } => {
                // widget 本地坐标 → canvas 原点平移（越界面出 canvas
                // 自动裁剪）。
                let at = iced::Point::new(rect.x, rect.y);
                frame.fill_rectangle(
                    at,
                    iced::Size::new(rect.w, rect.h),
                    to_color(*color),
                );
                i += 1;
            }
            DrawOp::Text { x, y, size, line_height, color, text } => {
                frame.fill_text(Text {
                    content: text.clone(),
                    position: iced::Point::new(*x, *y),
                    color: to_color(*color),
                    size: (*size).into(),
                    line_height: iced::widget::text::LineHeight::Absolute(
                        (*line_height).into(),
                    ),
                    ..Default::default()
                });
                i += 1;
            }
            // Plan 515 G2 —— typography 差分：weight/style 映射 iced Font
            //（宿主字体栈按 face 选择——cosmic-text 家族回退取最接近档）。
            DrawOp::TextStyled { x, y, size, line_height, color, weight, italic, text } => {
                frame.fill_text(Text {
                    content: text.clone(),
                    position: iced::Point::new(*x, *y),
                    color: to_color(*color),
                    size: (*size).into(),
                    line_height: iced::widget::text::LineHeight::Absolute(
                        (*line_height).into(),
                    ),
                    font: iced::Font {
                        weight: css_weight_to_iced(*weight),
                        style: if *italic {
                            iced::font::Style::Italic
                        } else {
                            iced::font::Style::Normal
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                });
                i += 1;
            }
        }
    }
}

/// queue 臂内容：DrawList → canvas 元素（Fill×Fill 客户区）。
pub fn drawlist_element(list: &DrawList) -> iced::Element<'_, DesktopMessage> {
    iced::widget::canvas(
        DrawListPainter { list: list.clone() },
    )
    .width(iced::Length::Fill)
    .height(iced::Length::Fill)
    .into()
}

/// independent 臂内容：RGBA 前缓冲 → Image（`from_rgba` 直接纳 straight
/// 非预乘；预乘换算在 iced 渲染器内部，协议层不感知）。
pub fn pixels_element(surface: &PixelsSurface) -> iced::Element<'_, DesktopMessage> {
    let handle = iced::widget::image::Handle::from_rgba(
        surface.w,
        surface.h,
        surface.rgba.clone(),
    );
    iced::widget::image(handle)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
}

/// 虚拟窗 wid → broker 表面内容（非 broker 窗 = None，调用方走本地
/// dynamic_view 既有路径）。
pub fn broker_client_content(
    state: &DesktopSession,
    wid: Wid,
) -> Option<iced::Element<'_, DesktopMessage>> {
    let client = state
        .broker_clients
        .values()
        .find(|c| c.wid == Some(wid))?;
    // 像素前缓冲优先（independent 臂），回退命令帧（queue 臂）。
    if let Some(px) = client.composed_pixels() {
        return Some(pixels_element(px));
    }
    client.composed().map(drawlist_element)
}

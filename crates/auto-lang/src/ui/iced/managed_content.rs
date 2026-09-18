//! PLAN-656 T-06: synthetic managed scroll content 的 iced widget。
//!
//! 架构角色（plan r2 §9.2/§11）：iced ManagedScrollBridge 的内容侧——
//! layout 声明**逻辑 extent**（scrollbar 比例由逻辑值驱动，§11.3-1），
//! draw 只物化可见窗口 ~20 行/列（§11.3-8），并在 draw 期观察 iced 传入的
//! viewport（内容坐标可见区）完成两条 hosting 通道：
//!
//! - `viewport_changed`：viewport 尺寸（含 mount/resize，§11.3-5）；
//! - `apply_scroll_intent(ScrollTo)`：用户滚动 offset 回灌 host（§11.3-3；
//!   terminal `virtual_scroll` draw 期观察同款；host 应用同值幂等，无回环）。
//!
//! controller 链路（§11.3-2/4）：controller intent → pane 排空 → iced
//! scroll_to → 下一帧 draw 观察 → host 同步——pane visual 与 host offset
//! 同源收敛。

use std::fmt::Debug;
use std::sync::Mutex;

use iced::advanced::layout::{Layout, Node};
use iced::advanced::text::Renderer as _TextRenderer;
use iced::advanced::widget::Tree;
use iced::advanced::widget::Widget;
use iced::advanced::renderer;
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::{mouse, Background, Border, Color, Element, Event, Length, Point, Rectangle, Size, Theme};

use crate::ui::scroll::host::ScrollContentHost;
use crate::ui::scroll::intent::{ScrollIntent, ScrollSource};
use crate::ui::scroll::managed::managed_host;
use crate::ui::scroll::state::ScrollViewportState;

/// 行高/列宽（逻辑 px，capability 验证固定值）。
const ROW_H: f64 = 40.0;
const COL_W: f64 = 160.0;
/// host viewport 未知时的可见窗口回退（首帧 on_scroll 未达）。
const FALLBACK_VIEWPORT_H: f64 = 800.0;
const FALLBACK_VIEWPORT_W: f64 = 600.0;
/// 单帧最多物化的行/列数（护栏：viewport 异常巨值不放大绘制）。
const MAX_ROWS_PER_FRAME: u64 = 64;
const MAX_COLS_PER_FRAME: u64 = 64;

pub struct ManagedScrollContentWidget {
    key: String,
    logical_w: f64,
    logical_h: f64,
}

impl ManagedScrollContentWidget {
    pub fn new(key: impl Into<String>, logical_w: f64, logical_h: f64) -> Self {
        Self { key: key.into(), logical_w, logical_h }
    }

    /// 直接构造 iced Element（renderer 臂消费）。
    pub fn into_element<M: Clone + Debug + 'static>(self) -> Element<'static, M> {
        Element::new(self)
    }
}

fn host_of(key: &str) -> std::sync::Arc<Mutex<crate::ui::scroll::host::SyntheticManagedContent>> {
    // axes 只在创建时生效；桥接面恒 BOTH（pane 的实际轴由外层 scrollable
    // Direction 决定，host 双轴状态独立存在）。
    managed_host(key, crate::ui::scroll::ScrollAxes::BOTH)
}

impl<M: Clone + Debug + 'static> Widget<M, Theme, iced::Renderer> for ManagedScrollContentWidget {
    fn size(&self) -> Size<Length> {
        // 逻辑 extent 即布局尺寸（10M < f32 精确整数域 2^24）。
        Size {
            width: Length::Fixed(self.logical_w as f32),
            height: Length::Fixed(self.logical_h as f32),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> Node {
        let w = Length::Fixed(self.logical_w as f32);
        let h = Length::Fixed(self.logical_h as f32);
        let limits = limits.width(w).height(h);
        Node::new(limits.resolve(w, h, Size::default()))
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        // draw 期 viewport 观察（terminal virtual_scroll 同款）：iced 传入的
        // viewport 是内容坐标可见区——y 差即滚动 offset，尺寸即视口测量。
        let offset_x = (viewport.x - bounds.x).max(0.0) as f64;
        let offset_y = (viewport.y - bounds.y).max(0.0) as f64;
        let vp_w = viewport.width.max(0.0) as f64;
        let vp_h = viewport.height.max(0.0) as f64;

        // 双通道回灌 host（同值幂等，无回环）。
        let host_arc = host_of(&self.key);
        {
            let mut host = host_arc.lock().unwrap();
            host.viewport_changed(ScrollViewportState { width: vp_w, height: vp_h });
            host.apply_scroll_intent(ScrollIntent::ScrollTo {
                axis: crate::ui::scroll::Axis::X,
                offset: offset_x,
                source: ScrollSource::Sync,
            });
            host.apply_scroll_intent(ScrollIntent::ScrollTo {
                axis: crate::ui::scroll::Axis::Y,
                offset: offset_y,
                source: ScrollSource::Sync,
            });
        }

        // 全幅底色。
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border::default(),
                ..renderer::Quad::default()
            },
            Background::Color(Color::from_rgb(0.07, 0.09, 0.11)),
        );

        // 可见行/列窗口（host 单源 offset；首帧 viewport 未达时回退观察值）。
        let (host_off_x, host_off_y, host_vw, host_vh) = {
            let h = host_arc.lock().unwrap();
            let (ox, oy) = h.host_offset();
            let vp = h.latest_viewport();
            (ox, oy, vp.width, vp.height)
        };
        let eff_vh = if host_vh > 0.0 { host_vh } else { FALLBACK_VIEWPORT_H };
        let eff_vw = if host_vw > 0.0 { host_vw } else { FALLBACK_VIEWPORT_W };
        let row0 = (host_off_y / ROW_H).floor().max(0.0) as u64;
        let rows = ((eff_vh / ROW_H).ceil() as u64 + 2).min(MAX_ROWS_PER_FRAME);
        let col0 = (host_off_x / COL_W).floor().max(0.0) as u64;
        let cols = ((eff_vw / COL_W).ceil() as u64 + 2).min(MAX_COLS_PER_FRAME);

        for i in row0..row0.saturating_add(rows) {
            let y = i as f64 * ROW_H;
            if y + ROW_H < host_off_y || y > host_off_y + eff_vh {
                continue;
            }
            let row_bounds = Rectangle::new(
                Point::new(bounds.x, bounds.y + y as f32),
                Size::new(bounds.width, ROW_H as f32),
            );
            let zebra = if i % 2 == 0 { Color::from_rgba(1.0, 1.0, 1.0, 0.03) } else { Color::TRANSPARENT };
            if zebra != Color::TRANSPARENT {
                renderer.fill_quad(
                    renderer::Quad { bounds: row_bounds, border: Border::default(), ..renderer::Quad::default() },
                    Background::Color(zebra),
                );
            }
            renderer.fill_text(
                iced::advanced::text::Text {
                    content: format!("row {} · y={}", i, y as u64),
                    bounds: Size::new(f32::INFINITY, f32::INFINITY),
                    size: 14.0.into(),
                    line_height: Default::default(),
                    font: Default::default(),
                    align_x: iced::alignment::Horizontal::Left.into(),
                    align_y: iced::alignment::Vertical::Top,
                    shaping: Default::default(),
                    wrapping: Default::default(),
                },
                Point::new(row_bounds.x + 8.0, row_bounds.y + 9.0),
                Color::from_rgb(0.75, 0.82, 0.88),
                *viewport,
            );
        }

        // 列标（双轴 case：x 轴物化窗口）。
        for j in col0..col0.saturating_add(cols) {
            let x = j as f64 * COL_W;
            if x + COL_W < host_off_x || x > host_off_x + eff_vw {
                continue;
            }
            renderer.fill_text(
                iced::advanced::text::Text {
                    content: format!("col {}", j),
                    bounds: Size::new(f32::INFINITY, f32::INFINITY),
                    size: 12.0.into(),
                    line_height: Default::default(),
                    font: Default::default(),
                    align_x: iced::alignment::Horizontal::Left.into(),
                    align_y: iced::alignment::Vertical::Top,
                    shaping: Default::default(),
                    wrapping: Default::default(),
                },
                Point::new(bounds.x + x as f32 + 6.0, bounds.y + host_off_y as f32 + 4.0),
                Color::from_rgb(0.45, 0.62, 0.9),
                *viewport,
            );
        }
    }

}

impl<M: Clone + Debug + 'static> From<ManagedScrollContentWidget> for Element<'static, M> {
    fn from(w: ManagedScrollContentWidget) -> Self {
        Element::new(w)
    }
}

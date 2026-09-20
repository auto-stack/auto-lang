//! PLAN-655: `items-stretch` 行的两阶段等高布局控件——CSS `align-items:stretch`
//! 语义原语化。
//!
//! 背景（P642-D12）：build_row 旧实现用「子项包 height:Fill 容器」模拟等高
//! 拉伸，Fill 需有界参照高；scroll 内容臂（无界）下 iced 0.14 flex 对
//! cross-Fill 子项走 second-pass 配给（max 从 `cross=0.0` 起步，见
//! iced_core layout/flex.rs second pass）→ 行高塌缩 0、008 定价卡整列消失。
//! 对「Fill 拉伸产物」做 compression 度量同样无效（Scrollable 的
//! compression 手法仅在压缩轴=主轴时成立，stretch 行的 Fill 在交叉轴）。
//!
//! 故 StretchLine 自持两遍行布局、直接持有**原始子项**（不经 Fill 包装）：
//! - **measure 遍**：全部子项在 `(可用宽, ∞)` + 子项主轴 compression 下取
//!   自然尺寸 → 行内容高 `h_line = max(子项自然高)`；
//! - **final 遍**：`effective = 有界父上下文 min(h_line, 入射上限)，无界取
//!   h_line`；交叉轴对 Shrink 高子项以 `min_h = effective` 落位（CSS
//!   stretch：auto 高 flex 项拉伸到行高；Fixed 高保持自然钳制；Fill 高经
//!   max 解析到 effective）；主轴按 flex third-pass 数学配给（Fill/FillPortion
//!   子项均分剩余宽——justify 垫片即 width-FillPortion Space，天然并入）。
//!
//! 行高语义与 Vue/CSS 臂同构：行高 auto = 最高子项内容高，子项等高 =
//! 行高；行自身定高（`h-full`/`flex-1` 转写/`h-*` Fixed，build_row 经
//! `fill_height` 传入）且有界入射时行高 = 入射上限（CSS definite-height
//! flex row 的 stretch——flex-grow 已把父列剩余空间分配给行，子项拉伸到
//! 该行高；无界入射回落 auto，P642-D12 不回归）。自身无状态、无自绘，
//! 事件/绘制/操作/hover 探针全部委托子项。

use iced::advanced::layout::{self, Layout, Limits};
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::event::Event;
use iced::{Element, Length, Rectangle, Size, Vector};

/// 构造等高行控件（build_row 的 `items_stretch` 分支专用；children 为
/// 「原始子项 + justify 垫片」的最终有序序列，spacing = 行 gap）。
/// `fill_height` = 行自身定高（style 带非 Shrink 的显式高，含 flex-1 在
/// Column 直接子位被 axis_fix_col_child 转写的 Height(Full)）。
pub fn stretch_line<Message: Clone + 'static>(
    children: Vec<Element<'static, Message>>,
    spacing: f32,
    fill_height: bool,
) -> StretchLine<'static, Message> {
    StretchLine { children, spacing, fill_height }
}

pub struct StretchLine<'a, Message> {
    children: Vec<Element<'a, Message>>,
    spacing: f32,
    /// 行自身定高：true 且入射上限有界 → 行高 = 入射上限（flex-grow 分配
    /// 的确定空间吃满，子项随之拉伸）；false → 行高 auto 语义（内容高）。
    fill_height: bool,
}

/// 行高解析：`fill_height` 且入射上限有界 → 上限（定高行吃满 flex-grow
/// 分配空间）；否则 auto 语义——内容高，仅被有界上限压矮（无界保持内容高，
/// scroll 内容臂不塌缩也不越权撑满）。纯函数便于单测。
fn line_effective_height(fill_height: bool, natural: f32, max_h: f32) -> f32 {
    if fill_height && max_h.is_finite() {
        max_h
    } else if max_h.is_finite() {
        natural.min(max_h)
    } else {
        natural
    }
}

/// 主轴配给份额：Fill 记 1 份，FillPortion(n) 记 n 份，其余 0（flex
/// fill_factor 同口径）。
fn fill_portion(length: Length) -> f32 {
    match length {
        Length::Fill => 1.0,
        Length::FillPortion(p) => p as f32,
        _ => 0.0,
    }
}

impl<Message: Clone + 'static> Widget<Message, iced::Theme, iced::Renderer>
    for StretchLine<'_, Message>
{
    fn size(&self) -> Size<Length> {
        Size { width: Length::Shrink, height: Length::Shrink }
    }

    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let n = self.children.len();
        if n == 0 {
            return layout::Node::new(Size::ZERO);
        }
        let max = limits.max();
        let avail_w = max.width.max(0.0);
        let total_spacing = self.spacing * (n - 1) as f32;

        // —— 宽度探测遍：主轴配给份额（非 fill 子项按序吃自然宽，fill 子项
        // 均分剩余——flex first/third pass 同式）。份额宽必须先定：文本换行
        // 行数依赖最终宽，测高用错宽会把最高卡测短（内容下溢钳裁）。
        let mut available = (avail_w - total_spacing).max(0.0);
        let fill_sum: f32 = self
            .children
            .iter()
            .map(|c| fill_portion(c.as_widget().size().width))
            .sum();
        let mut final_w = vec![0.0f32; n];
        for (i, child) in self.children.iter_mut().enumerate() {
            if fill_portion(child.as_widget().size().width) == 0.0 {
                let child_limits = Limits::with_compression(
                    Size::ZERO,
                    Size::new(available.max(0.0), f32::INFINITY),
                    Size::new(false, false),
                );
                let node = child
                    .as_widget_mut()
                    .layout(&mut tree.children[i], renderer, &child_limits);
                let w = node.size().width;
                available -= w;
                final_w[i] = w;
            }
        }
        let remaining = available.max(0.0);
        for (i, child) in self.children.iter().enumerate() {
            let portion = fill_portion(child.as_widget().size().width);
            if portion > 0.0 {
                final_w[i] = if fill_sum > 0.0 {
                    (remaining * portion / fill_sum).max(0.0)
                } else {
                    0.0
                };
            }
        }

        // —— measure 遍：按各自最终份额宽测内容自然高。compression 打在
        // **子项 flex 的主轴**上（子项列垂直 → compression.height=true）：
        // justify-between 列的 FillPortion 垫片在 main-compress 下解析为
        // 内容高 0（不再吸走 ∞ 剩余高），列高 = 纯内容高；Scrollable 同款
        // 手法（009/016 实证）。注意不可用 (false,false)——垫片 third-pass
        // 会把列高抬到入射上限。
        let mut natural_h = 0.0f32;
        for (i, child) in self.children.iter_mut().enumerate() {
            let child_limits = Limits::with_compression(
                Size::ZERO,
                Size::new(final_w[i], f32::INFINITY),
                Size::new(false, true),
            );
            let node = child
                .as_widget_mut()
                .layout(&mut tree.children[i], renderer, &child_limits);
            natural_h = natural_h.max(node.size().height);
        }
        let effective = line_effective_height(self.fill_height, natural_h, max.height);

        // —— final 遍：按份额宽落位 + 交叉轴拉伸。
        let mut nodes: Vec<layout::Node> = (0..n).map(|_| layout::Node::default()).collect();
        for (i, child) in self.children.iter_mut().enumerate() {
            // 交叉轴拉伸：Shrink 高子项 min=effective（CSS stretch 载体，
            // 同时让 justify-between 列在行高内分布内容）；Fixed 高不拉伸
            // （CSS：显式高 flex 项不参与 stretch），自然钳制到 effective；
            // Fill 高经 max 解析到 effective。
            let min_h = if child.as_widget().size().height == Length::Shrink {
                effective
            } else {
                0.0
            };
            let child_limits = Limits::with_compression(
                Size::new(0.0, min_h),
                Size::new(final_w[i], effective),
                Size::new(false, false),
            );
            nodes[i] =
                child.as_widget_mut().layout(&mut tree.children[i], renderer, &child_limits);
        }

        // 定位：顺序排列 + spacing（行高 = effective，子项顶对齐——拉伸后
        // 天然等高，无对齐余量）。
        let mut x = 0.0f32;
        for node in nodes.iter_mut() {
            node.translate_mut(Vector::new(x, 0.0));
            x += node.size().width + self.spacing;
        }
        let total_w = (x - self.spacing).max(0.0);
        layout::Node::with_children(Size::new(total_w, effective), nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        for (i, child) in self.children.iter_mut().enumerate() {
            if let Some(cl) = layout.children().nth(i) {
                child.as_widget_mut().operate(&mut tree.children[i], cl, renderer, operation);
            }
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let child_layouts: Vec<Layout<'_>> = layout.children().collect();
        for (i, child) in self.children.iter_mut().enumerate() {
            // PLAN-063 对称防帘：tree 落后于布局子件数时降级跳过，不 panic。
            if let (Some(cl), true) = (child_layouts.get(i), i < tree.children.len()) {
                child.as_widget_mut().update(
                    &mut tree.children[i],
                    event,
                    *cl,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let mut best = mouse::Interaction::None;
        for (i, child) in self.children.iter().enumerate() {
            if let Some(cl) = layout.children().nth(i) {
                best = best.max(child.as_widget().mouse_interaction(
                    &tree.children[i],
                    cl,
                    cursor,
                    viewport,
                    renderer,
                ));
            }
        }
        best
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for (i, child) in self.children.iter().enumerate() {
            if let Some(cl) = layout.children().nth(i) {
                child.as_widget().draw(
                    &tree.children[i],
                    renderer,
                    theme,
                    inherited_style,
                    cl,
                    cursor,
                    viewport,
                );
            }
        }
    }
}

impl<'a, Message: Clone + 'static> From<StretchLine<'a, Message>>
    for Element<'a, Message>
{
    fn from(line: StretchLine<'a, Message>) -> Self {
        Element::new(line)
    }
}

#[cfg(test)]
mod tests {
    use super::line_effective_height as eff;

    // auto 语义：行高 = 内容高，仅被有界上限压矮。
    #[test]
    fn auto_row_keeps_content_height() {
        assert_eq!(eff(false, 400.0, f32::INFINITY), 400.0);
        assert_eq!(eff(false, 400.0, 1100.0), 400.0);
        assert_eq!(eff(false, 1500.0, 1100.0), 1100.0);
    }

    // 定高行：有界入射吃满上限（flex-grow 分配空间）；无界回落 auto。
    #[test]
    fn definite_row_fills_bounded_limits() {
        assert_eq!(eff(true, 400.0, 1100.0), 1100.0);
        assert_eq!(eff(true, 1500.0, 1100.0), 1100.0);
        assert_eq!(eff(true, 400.0, f32::INFINITY), 400.0);
    }
}

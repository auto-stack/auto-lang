//! PLAN-082 T-05: 表格自然宽列对齐 widget——`View::Table` 自然宽臂（无
//! oncolresize 通道）的 iced 承载。
//!
//! 为什么：既有自然宽 lowering 是 column of (header row + rule + body row
//! + rule …)，每行独立 Row、cell 按内容 hug——行与行之间列 x 不对齐
//! （web 侧真 `<table>` 有统一列宽算法；VM 截图实证列错位）。本 widget
//! 自持整表布局：第一遍 loose limits 测每列自然宽（该列 header+body 全部
//! cell 取 max，含 CELL_PAD），第二遍按列宽 Fixed 落格——跨行严格对齐；
//! 行分隔线 draw 自绘（table_row_rule 同色同高、全容器宽同旧 Fill 行为）。
//! 列宽合计超容器可用宽时按比例压缩（下限 40px，[`MIN_COL_WIDTH`] 同源），
//! cell 内文本随之定宽折行（与 web table-layout auto 的折行同向）。
//!
//! oncolresize 通道存在时仍分派 [`super::table_resize::table_resize`]，
//! 本 widget 只接管自然宽臂（结构/常量与 table_resize.rs 同源）。

use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::event::Event;
use iced::{Element, Length, Point, Rectangle, Size, Vector};

/// cell 内边距（px）——renderer::table_cell_container 的 px-4/py-3 同源。
const CELL_PAD_X: f32 = 16.0;
const CELL_PAD_Y: f32 = 12.0;
/// 行分隔线高（px）——renderer::table_row_rule 的 1px 同源。
const RULE_H: f32 = 1.0;
/// 压缩下的列宽下限（px）——clamp_col_width 的 40 同源。
const MIN_COL_WIDTH: f32 = 40.0;

pub struct TableAlign<'a, Message: 'static> {
    header_cells: Vec<Element<'a, Message>>,
    body_rows: Vec<Vec<Element<'a, Message>>>,
    col_spacing: f32,
}

/// 构造自然宽列对齐表格 widget（renderer Table 臂 col_widths 与
/// on_col_resize 皆 None 时分派至此）。
pub fn table_align<'a, Message: Clone + 'static>(
    header_cells: Vec<Element<'a, Message>>,
    body_rows: Vec<Vec<Element<'a, Message>>>,
    col_spacing: f32,
) -> TableAlign<'a, Message> {
    TableAlign { header_cells, body_rows, col_spacing }
}

/// 列宽压缩纯函数——合计超可用宽时按比例收缩（下限 MIN_COL_WIDTH；
/// 下限合计仍超则接受溢出，交外层滚动）。返回 None = 无需压缩。
fn compress_widths(natural: &[f32], avail_content: f32) -> Option<Vec<f32>> {
    let total: f32 = natural.iter().sum();
    if total <= avail_content || natural.is_empty() {
        return None;
    }
    let scale = avail_content / total;
    Some(natural.iter().map(|w| (w * scale).max(MIN_COL_WIDTH)).collect())
}

impl<'a, Message: 'static> Widget<Message, iced::Theme, iced::Renderer>
    for TableAlign<'a, Message>
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<Vec<Rectangle>>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(Vec::<Rectangle>::new())
    }

    fn children(&self) -> Vec<Tree> {
        let mut trees: Vec<Tree> = self.header_cells.iter().map(Tree::new).collect();
        for r in &self.body_rows {
            trees.extend(r.iter().map(Tree::new));
        }
        trees
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(
            &self.header_cells.iter().chain(self.body_rows.iter().flatten()).collect::<Vec<_>>(),
        );
    }

    fn size(&self) -> Size<Length> {
        Size { width: Length::Shrink, height: Length::Shrink }
    }

    fn size_hint(&self) -> Size<Length> {
        Size { width: Length::Shrink, height: Length::Shrink }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let n_cols = self
            .header_cells
            .len()
            .max(self.body_rows.iter().map(|r| r.len()).max().unwrap_or(0));
        if n_cols == 0 {
            return layout::Node::new(limits.resolve(
                Length::Shrink,
                Length::Shrink,
                Size::ZERO,
            ));
        }

        // —— 第一遍：measure 自然宽（loose 限宽，每列取 header+body 最大）——
        let mut natural_col = vec![0.0f32; n_cols];
        {
            let mut ci = 0usize;
            let cells = self
                .header_cells
                .iter_mut()
                .enumerate()
                .chain(self.body_rows.iter_mut().flat_map(|r| r.iter_mut().enumerate()));
            for (col, cell) in cells {
                let node = cell.as_widget_mut().layout(
                    &mut tree.children[ci],
                    renderer,
                    &limits.loose(),
                );
                natural_col[col.min(n_cols - 1)] =
                    natural_col[col.min(n_cols - 1)].max(node.size().width);
                ci += 1;
            }
        }
        let natural_full: Vec<f32> =
            natural_col.iter().map(|w| w + 2.0 * CELL_PAD_X).collect();

        // —— 容器宽约束：合计超可用按比例压缩（下限 40）——
        let avail_content =
            (limits.max().width - self.col_spacing * (n_cols - 1) as f32).max(0.0);
        let effective: Vec<f32> =
            compress_widths(&natural_full, avail_content).unwrap_or(natural_full);

        let col_x: Vec<f32> = {
            let mut xs = Vec::with_capacity(n_cols);
            let mut x = 0.0f32;
            for w in &effective {
                xs.push(x);
                x += w + self.col_spacing;
            }
            xs
        };
        let total_w = effective.iter().sum::<f32>()
            + self.col_spacing * n_cols.saturating_sub(1) as f32;

        // —— 第二遍：按列宽限宽落格（行分组：headers 一组 + 每 body 行一组）——
        let mut nodes: Vec<layout::Node> = Vec::new();
        let mut row_groups: Vec<(usize, usize)> = Vec::new();
        {
            let mut ci = 0usize;
            let mut lay_row =
                |cells: &mut dyn Iterator<Item = (usize, &mut Element<'_, Message>)>,
                 tree: &mut Tree,
                 nodes: &mut Vec<layout::Node>,
                 ci: &mut usize| {
                    let start = nodes.len();
                    for (col, cell) in cells {
                        let col = col.min(n_cols - 1);
                        let inner_w = (effective[col] - 2.0 * CELL_PAD_X).max(0.0);
                        let cell_limits = limits
                            .width(Length::Fixed(inner_w))
                            .height(Length::Shrink);
                        let node = cell.as_widget_mut().layout(
                            &mut tree.children[*ci],
                            renderer,
                            &cell_limits,
                        );
                        nodes.push(node);
                        *ci += 1;
                    }
                    start
                };
            let g0 = lay_row(&mut self.header_cells.iter_mut().enumerate(), tree, &mut nodes, &mut ci);
            row_groups.push((g0, self.header_cells.len()));
            for r in self.body_rows.iter_mut() {
                let s = lay_row(&mut r.iter_mut().enumerate(), tree, &mut nodes, &mut ci);
                row_groups.push((s, r.len()));
            }
        }

        // 行高 = 组内 cell 高最大 + 上下 padding；分隔线随行尾（全容器宽，
        // 同旧 table_row_rule 的 Fill 行为）。
        let mut rules: Vec<Rectangle> = Vec::new();
        let mut y = 0.0f32;
        let mut row_y: Vec<f32> = Vec::new();
        for (gi, &(start, count)) in row_groups.iter().enumerate() {
            let h = nodes[start..start + count]
                .iter()
                .map(|n| n.size().height + 2.0 * CELL_PAD_Y)
                .fold(0.0f32, f32::max);
            let _ = gi;
            row_y.push(y);
            y += h;
            rules.push(Rectangle::new(
                Point::new(0.0, y),
                Size::new(limits.max().width.max(total_w), RULE_H),
            ));
            y += RULE_H;
        }

        // 定位平移：cell 按列 x + 内边距、行 y + 上内边距。
        for (gi, &(start, count)) in row_groups.iter().enumerate() {
            for k in 0..count {
                let col = k.min(n_cols - 1);
                nodes[start + k].translate_mut(Vector::new(
                    col_x[col] + CELL_PAD_X,
                    row_y[gi] + CELL_PAD_Y,
                ));
            }
        }

        // 布局现场经 tree.state 携带给 draw（规则线本地坐标）。State=Vec<Rectangle>。
        *tree.state.downcast_mut::<Vec<Rectangle>>() = rules.clone();

        layout::Node::with_children(Size::new(total_w.max(limits.max().width), y), nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        // 转发 cell 子树（iced_test find / MCP 探针可达性，table_resize 同款）。
        let mut ci = 0usize;
        for cell in self.header_cells.iter_mut().chain(self.body_rows.iter_mut().flatten()) {
            if let Some(cl) = layout.children().nth(ci) {
                cell.as_widget_mut().operate(&mut tree.children[ci], cl, renderer, operation);
            }
            ci += 1;
        }
    }

    fn update(
        &mut self,
        _tree: &mut Tree,
        _event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        // 纯展示表：无交互。
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        // 无自身命中态，委托 cell 子树（文本光标等）。
        let mut best = mouse::Interaction::None;
        let mut ci = 0usize;
        for cell in self.header_cells.iter().chain(self.body_rows.iter().flatten()) {
            if let Some(cl) = layout.children().nth(ci) {
                best = best.max(cell.as_widget().mouse_interaction(
                    &tree.children[ci],
                    cl,
                    cursor,
                    viewport,
                    renderer,
                ));
            }
            ci += 1;
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
        let bounds = layout.bounds();

        let mut ci = 0usize;
        for cell in self.header_cells.iter().chain(self.body_rows.iter().flatten()) {
            if let Some(cl) = layout.children().nth(ci) {
                cell.as_widget().draw(
                    &tree.children[ci],
                    renderer,
                    theme,
                    inherited_style,
                    cl,
                    cursor,
                    viewport,
                );
            }
            ci += 1;
        }

        // 行分隔线（renderer::table_row_rule 同色同高，本地坐标 → 绝对）。
        let rules = tree.state.downcast_ref::<Vec<Rectangle>>();
        let (r, g, b) = crate::ui::style::iced_adapter::resolve_border_rgb();
        let rule_color = iced::Color::from_rgb8(r, g, b);
        for rect in rules {
            fill_quad(
                renderer,
                Rectangle::new(
                    Point::new(bounds.x + rect.x, bounds.y + rect.y),
                    rect.size(),
                ),
                rule_color,
            );
        }
    }
}

fn fill_quad(renderer: &mut iced::Renderer, rect: Rectangle, color: iced::Color) {
    renderer.fill_quad(
        renderer::Quad { bounds: rect, ..renderer::Quad::default() },
        iced::Background::Color(color),
    );
}

impl<'a, Message: Clone + 'static> From<TableAlign<'a, Message>> for Element<'a, Message> {
    fn from(w: TableAlign<'a, Message>) -> Self {
        Element::new(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_widths_scales_proportionally_with_floor() {
        // 无超宽：原样返回 None。
        assert_eq!(compress_widths(&[100.0, 200.0], 400.0), None);
        // 超宽：等比收缩；短列触底 40。
        let out = compress_widths(&[300.0, 300.0], 300.0).unwrap();
        assert_eq!(out, vec![150.0, 150.0]);
        let out = compress_widths(&[20.0, 400.0], 210.0).unwrap();
        // scale=0.5 → 10→40(floor)、200。
        assert_eq!(out, vec![40.0, 200.0]);
        // 空列安全。
        assert_eq!(compress_widths(&[], 100.0), None);
    }
}

//! Plan 045 T3: 表格列宽拖拽 widget——`View::Table::on_col_resize` 的 iced 承载。
//!
//! 为什么不用既有 lowering + PointerArea 包装：拖拽要求「列宽实时改」，
//! 而 cell 容器定宽在 view 构建期固化（into_iced 产物），包装层改不了
//! 内层布局。本 widget 自持表格网格布局（measure 自然宽 → 生效宽分派 →
//! 定位），拖拽临时宽写 [`State`]（tree 本地，不进 DSL state）+
//! `Shell::invalidate_layout` 实时重排（iced 0.14 API），松手才 publish
//! [`ColResizeCallback`] 落定消息——会话裁定「拖拽中临时宽实时绘制、松手
//! 才发消息」的落点，避免 mousemove 级消息洪泛 DSL 层。
//!
//! 常量口径（行为对齐 vue 金标 useTableColumnResize）：
//! - 命中带 10px（对称 ±5，[`COL_RESIZE_BAND`]）——金标 right-6~right+4；
//! - 最小宽 40（[`MIN_COL_WIDTH`]）——金标 `max(40, …)`；
//! - 宽度单位 px；指示线 2px 竖线（金标 body 挂 fixed 2px div）。
//!
//! 结构先例：PointerArea（tree::Tag 本地 State + 事件现场 layout bounds）、
//! code_editor 滚动条（Drag 态 + fill_quad 直绘）。

use crate::ui::view::{clamp_col_width, col_boundary_hit, ColResizeCallback, ColResizeMetrics, COL_RESIZE_BAND};
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
/// 拖拽/悬浮指示线宽（px）——vue 金标 body 挂 2px 竖线 div。
const INDICATOR_W: f32 = 2.0;

/// TableResize 的本地状态（tree::Tag 标识，跨帧持久、不进 DSL state）。
#[derive(Debug, Default)]
pub struct State {
    /// 最近一次 layout 的生效列宽（命中几何与拖拽 start_w 的单一事实源）。
    widths: Vec<f32>,
    /// 表头带高（命中区域上界）。
    header_h: f32,
    /// 拖拽态（code_editor 滚动条 Drag 同款）：col + 起点几何 + 当前临时宽。
    drag: Option<DragState>,
    /// 悬浮命中的列边界（指示线显示）。
    hover_col: Option<usize>,
    /// layout 现场写回的行分隔线（本地坐标，draw 直读）。
    rules: Vec<Rectangle>,
    /// 指示线 x（本地坐标；hover 或 drag 态）。
    indicator: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
struct DragState {
    col: usize,
    start_x: f32,
    start_w: f32,
    current_w: f32,
}

/// 拖拽临时宽纯函数——起点宽 + 位移，clamp 最小宽（vue 金标 L134 同式）。
fn drag_width(start_w: f32, start_x: f32, x: f32) -> f32 {
    clamp_col_width(start_w + (x - start_x))
}

pub struct TableResize<'a, Message: 'static> {
    header_cells: Vec<Element<'a, Message>>,
    body_rows: Vec<Vec<Element<'a, Message>>>,
    col_spacing: f32,
    applied_widths: Option<Vec<f32>>,
    on_resize: ColResizeCallback<Message>,
}

/// Plan 045 T3: 构造列宽拖拽表格 widget（renderer Table 臂 on_col_resize
/// Some 时分派至此；None 走既有 lowering，零回归）。
pub fn table_resize<'a, Message: Clone + 'static>(
    header_cells: Vec<Element<'a, Message>>,
    body_rows: Vec<Vec<Element<'a, Message>>>,
    col_spacing: f32,
    applied_widths: Option<Vec<f32>>,
    on_resize: ColResizeCallback<Message>,
) -> TableResize<'a, Message> {
    TableResize { header_cells, body_rows, col_spacing, applied_widths, on_resize }
}

impl<Message: Clone + 'static> Widget<Message, iced::Theme, iced::Renderer>
    for TableResize<'_, Message>
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        let mut trees: Vec<Tree> = self.header_cells.iter().map(Tree::new).collect();
        for r in &self.body_rows {
            trees.extend(r.iter().map(Tree::new));
        }
        trees
    }

    fn diff(&self, tree: &mut Tree) {
        let cells: Vec<&Element<Message>> =
            self.header_cells.iter().chain(self.body_rows.iter().flatten()).collect();
        tree.diff_children(&cells);
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

        // —— 第一遍：measure 自然宽（loose 限宽，Shrink 取内容宽）——
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

        // —— 生效宽分派：drag 临时宽 > applied 固定宽 > 自然宽 ——
        let effective: Vec<f32> = {
            let state = tree.state.downcast_mut::<State>();
            (0..n_cols)
                .map(|i| {
                    if let Some(d) = state.drag {
                        if d.col == i {
                            return clamp_col_width(d.current_w);
                        }
                    }
                    self.applied_widths
                        .as_ref()
                        .and_then(|ws| ws.get(i).copied())
                        .unwrap_or(natural_full[i])
                })
                .collect()
        };
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
        let mut row_groups: Vec<(usize /*起始索引*/, usize /*cell 数*/)> = Vec::new();
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

        // 行高 = 组内 cell 高最大 + 上下 padding；分隔线随行尾。
        let mut rules: Vec<Rectangle> = Vec::new();
        let mut y = 0.0f32;
        let mut row_y: Vec<f32> = Vec::new();
        let mut header_h = 0.0f32;
        for (gi, &(start, count)) in row_groups.iter().enumerate() {
            let h = nodes[start..start + count]
                .iter()
                .map(|n| n.size().height + 2.0 * CELL_PAD_Y)
                .fold(0.0f32, f32::max);
            if gi == 0 {
                header_h = h;
            }
            row_y.push(y);
            y += h;
            rules.push(Rectangle::new(Point::new(0.0, y), Size::new(total_w, RULE_H)));
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

        // —— 状态写回（命中几何 + 绘制现场）——
        {
            let state = tree.state.downcast_mut::<State>();
            state.header_h = header_h;
            state.widths = effective.clone();
            state.rules = rules;
            state.indicator = if let Some(d) = state.drag {
                Some(col_x[d.col.min(n_cols - 1)] + effective[d.col.min(n_cols - 1)])
            } else {
                state.hover_col
                    .map(|c| col_x[c.min(n_cols - 1)] + effective[c.min(n_cols - 1)])
            };
        }

        layout::Node::with_children(Size::new(total_w, y), nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        // 转发 cell 子树（iced_test find / snapshot 探针可达性）。
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
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        // 事件现场坐标（PointerArea 先例）：CursorMoved 用事件自带全局位
        //（拖拽出界仍连续），其余用 runtime cursor。
        let local: Option<(f32, f32)> = match event {
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                Some((position.x - bounds.x, position.y - bounds.y))
            }
            _ => cursor
                .position()
                .map(|p| (p.x - bounds.x, p.y - bounds.y)),
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let state = tree.state.downcast_mut::<State>();
                if let Some((x, y)) = local {
                    if y >= 0.0 && y <= state.header_h {
                        if let Some(col) =
                            col_boundary_hit(x, &state.widths, self.col_spacing, COL_RESIZE_BAND)
                        {
                            let start_w = state.widths.get(col).copied().unwrap_or_default();
                            state.drag = Some(DragState {
                                col,
                                start_x: x,
                                start_w,
                                current_w: start_w,
                            });
                            state.hover_col = Some(col);
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let state = tree.state.downcast_mut::<State>();
                if let Some((x, y)) = local {
                    if let Some(d) = &mut state.drag {
                        // 拖拽中：临时宽实时重排（不进 DSL state——会话裁定）。
                        d.current_w = drag_width(d.start_w, d.start_x, x);
                        shell.invalidate_layout();
                    } else if cursor.is_over(bounds)
                        && y >= 0.0
                        && y <= state.header_h
                        && x >= 0.0
                    {
                        let hit =
                            col_boundary_hit(x, &state.widths, self.col_spacing, COL_RESIZE_BAND);
                        if hit != state.hover_col {
                            state.hover_col = hit;
                            shell.invalidate_layout();
                        }
                    } else if state.hover_col.take().is_some() {
                        shell.invalidate_layout();
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let state = tree.state.downcast_mut::<State>();
                if let Some(d) = state.drag.take() {
                    // 松手才发落定消息（clamp min 40 在 drag_width 内）。
                    shell.publish(self.on_resize.call(ColResizeMetrics {
                        col: d.col,
                        width: clamp_col_width(d.current_w),
                    }));
                    shell.invalidate_layout();
                }
                state.hover_col = None;
            }
            _ => {}
        }

        // 事件转发给 cell 子树（不吞事件——PointerArea 同语义；cell 为
        // 只读文本，转发保持 selection 等既有行为）。
        let child_layouts: Vec<Layout<'_>> = layout.children().collect();
        let mut ci = 0usize;
        let mut forward = |cells: &mut dyn Iterator<Item = &mut Element<'_, Message>>,
                           tree: &mut Tree,
                           ci: &mut usize| {
            for cell in cells {
                if let Some(cl) = child_layouts.get(*ci) {
                    cell.as_widget_mut().update(
                        &mut tree.children[*ci],
                        event,
                        *cl,
                        cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );
                }
                *ci += 1;
            }
        };
        forward(&mut self.header_cells.iter_mut(), tree, &mut ci);
        for r in self.body_rows.iter_mut() {
            forward(&mut r.iter_mut(), tree, &mut ci);
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
        let state = tree.state.downcast_ref::<State>();
        if state.drag.is_some() || state.hover_col.is_some() {
            return mouse::Interaction::ResizingColumn;
        }
        // 无命中时委托 cell 子树（文本光标等）。
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
        let state = tree.state.downcast_ref::<State>();
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
        let (r, g, b) = crate::ui::style::iced_adapter::resolve_border_rgb();
        let rule_color = iced::Color::from_rgb8(r, g, b);
        for rect in &state.rules {
            fill_quad(
                renderer,
                Rectangle::new(
                    Point::new(bounds.x + rect.x, bounds.y + rect.y),
                    rect.size(),
                ),
                rule_color,
            );
        }

        // 拖拽/悬浮指示线（2px 竖线，vue 金标 body 挂 fixed div 同形）。
        if let Some(x) = state.indicator {
            let (pr, pg, pb) = crate::ui::style::theme::resolve_semantic_rgb(
                &crate::ui::style::Color::Primary,
            )
            .unwrap_or((99, 102, 241));
            fill_quad(
                renderer,
                Rectangle::new(
                    Point::new(bounds.x + x - INDICATOR_W / 2.0, bounds.y),
                    Size::new(INDICATOR_W, bounds.height),
                ),
                iced::Color::from_rgb8(pr, pg, pb),
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

impl<'a, Message: Clone + 'static> From<TableResize<'a, Message>> for Element<'a, Message> {
    fn from(w: TableResize<'a, Message>) -> Self {
        Element::new(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_width_clamps_to_min() {
        // 拖拽临时宽：正向跟随、负向 clamp 到 40（vue 金标 max(40,…)）。
        assert_eq!(drag_width(100.0, 200.0, 250.0), 150.0);
        assert_eq!(drag_width(100.0, 200.0, 200.0), 100.0);
        assert_eq!(drag_width(100.0, 200.0, 50.0), 40.0);
        assert_eq!(drag_width(100.0, 200.0, -1000.0), 40.0);
    }
}

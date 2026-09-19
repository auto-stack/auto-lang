//! # Scroll State Reader Operation（PLAN-656 review F-4）
//!
//! 自定义 `iced::advanced::widget::Operation`：遍历 scrollable 树读回
//! **真实** offset/viewport/content_extent 六测量（id → 六元组），经
//! `__scroll_state_read` 消息回 update 层写入 controller 注册表。
//!
//! 动机：iced 程序化 `scroll_to` 不触发 `on_scroll` 回声，且 pane 无用户
//! 滚动前注册表无任何测量——controller 首用（to_end 等）对全零快照解析
//! 恒得 0。读回通道两处消费：①bind 期预热请求（新绑定 handle 下一 tick
//! 补测量基线）；②controller intent 排空后校正（scroll_to 落盘的真实
//! clamp 结果写回投影）。
//!
//! offset 来源：operation 钩子的 `translation` 向量（内容平移 = 取负即
//! 滚动 offset，向下/右滚为正，clamp ≥ 0）。

use std::collections::HashMap;

use iced::advanced::widget::Operation;
use iced::advanced::widget::operation::{Outcome, Scrollable};
use iced::{Rectangle, Vector};
use iced::widget::Id;

/// 读回结果：widget id 串 → (offset_x, offset_y, viewport_w, viewport_h, content_w, content_h)。
pub type ScrollStateMap = HashMap<String, (f32, f32, f32, f32, f32, f32)>;

/// 从 iced `Id` 的 Debug 形态（`Id(Custom("scroll_ctl_main"))`）提取内部串。
fn id_key(id: &Id) -> String {
    let debug = format!("{id:?}");
    if let Some(q1) = debug.find('"') {
        let rest = &debug[q1 + 1..];
        let end = rest.find('"').unwrap_or(rest.len());
        return rest[..end].to_string();
    }
    debug
}

pub struct ScrollStateReader {
    out: ScrollStateMap,
}

impl ScrollStateReader {
    pub fn new() -> Self {
        Self { out: HashMap::new() }
    }
}

impl Default for ScrollStateReader {
    fn default() -> Self {
        Self::new()
    }
}

impl Operation<ScrollStateMap> for ScrollStateReader {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<ScrollStateMap>)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        _state: &mut dyn Scrollable,
    ) {
        if let Some(id) = id {
            let offset_x = (-translation.x).max(0.0);
            let offset_y = (-translation.y).max(0.0);
            self.out.insert(
                id_key(id),
                (
                    offset_x,
                    offset_y,
                    bounds.width.max(0.0),
                    bounds.height.max(0.0),
                    content_bounds.width.max(0.0),
                    content_bounds.height.max(0.0),
                ),
            );
        }
    }

    fn finish(&self) -> Outcome<ScrollStateMap> {
        Outcome::Some(self.out.clone())
    }
}

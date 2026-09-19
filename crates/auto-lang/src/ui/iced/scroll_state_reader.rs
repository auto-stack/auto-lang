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
//! clamp 结果写回投影——v1 消费端为 extent-only，见 renderer 侧注记）。
//!
//! offset 来源：operation 钩子的 `translation` 向量。**iced 0.14 符号约定：
//! translation 即正向滚动 offset**（`State::translation` 与
//! `Viewport::absolute_offset` 同源同号，向下/右滚为正；draw 侧以
//! `-translation` 平移内容层）。F-4 末环曾按"内容平移取负"解读写成
//! `(-translation).max(0)`——任何正向 offset 都被 clamp 成 0，是当轮
//! "translation 读回恒 0 / 重建重置 offset"误判的唯一来源（obs pane 无
//! controller/写臂，跨多次重建仍保持滚动位，截图实证 offset 持久）。

use std::collections::HashMap;

use iced::advanced::widget::operation::{Outcome, Scrollable};
use iced::advanced::widget::Operation;
use iced::widget::Id;
use iced::{Rectangle, Vector};

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
        Self {
            out: HashMap::new(),
        }
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
            // iced 0.14：translation 与 absolute_offset 同号（正向滚动为正）；
            // 负值（overscroll 回弹瞬态）clamp 到 0。
            let offset_x = translation.x.max(0.0);
            let offset_y = translation.y.max(0.0);
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
